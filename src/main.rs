#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crossbeam;
use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use rand::seq::SliceRandom;
use rand::Rng;
use seahash::SeaHasher;
use std::{hash::BuildHasherDefault, mem, sync::{RwLock, mpsc::{channel, Sender}}, time::Instant};

#[derive(Debug)]
struct Creature {
    key_area: (usize, usize),
    to_remove: bool,
    param: usize,
}

impl Creature {
    pub fn new(key_area: (usize, usize)) -> Creature {
        Creature {
            key_area,
            to_remove: false,
            param: 0,
        }
    }
}

#[derive(Debug)]
struct Area {
    key_creature: Option<usize>,
    vacant: bool,
    param: usize,
    smth: f64,
}

impl Area {
    pub fn new() -> Area {
        Area {
            key_creature: None,
            vacant: true,
            param: 0,
            smth: 123.123,
        }
    }
}

type TAreas = IndexMap<(usize, usize), RwLock<Area>, BuildHasherDefault<SeaHasher>>;

struct World {
    size_x: usize,
    size_y: usize,
    areas: TAreas,
}

impl World {
    pub fn new(areas: TAreas) -> World {
        World {
            size_x: 0,
            size_y: 0,
            areas,
        }
    }
}

type TCreatures = IndexMap<usize, RwLock<Creature>, BuildNoHashHasher<usize>>;

struct Population {
    creatures: TCreatures,
}

impl Population {
    pub fn new(creatures: TCreatures) -> Population {
        Population { creatures }
    }
}

fn fill_world_population(
    world: &mut World,
    population: &mut Population,
    world_size: (usize, usize),
    pop_size: usize,
) {
    // Fill world with areas
    print!("Create areas: ");
    let t0 = Instant::now();
    world.size_x = world_size.0;
    world.size_y = world_size.1;
    for x in 0..world_size.0 {
        for y in 0..world_size.1 {
            world.areas.insert((x, y), RwLock::new(Area::new()));
        }
    }
    println!("{:.3} sec", t0.elapsed().as_secs_f64());

    // Create creatures in areas
    print!("Create population: ");
    let t0 = Instant::now();
    let mut rng = rand::thread_rng();
    let mut world_keys = world.areas.keys().copied().collect::<Vec<(usize, usize)>>();
    world_keys.shuffle(&mut rng);
    world_keys.truncate(pop_size);
    let mut irange = 0..world_keys.len();
    for k in world_keys {
        let i = irange.next().unwrap();
        let a = world.areas.get_mut(&k).unwrap().get_mut().unwrap();
        let m_opt = population
            .creatures
            .insert(i, RwLock::new(Creature::new(k)));
        match m_opt {
            Some(c) => panic!("Area already contains creature {:?}", c),
            None => {
                a.key_creature = Some(i);
                a.vacant = false;
            }
        }
    }
    println!("{:.3} sec", t0.elapsed().as_secs_f64());
}

fn threads_processing(world: &mut World, population: &mut Population, free_creature_keys: &mut Vec<usize>, pop_chunk_size: usize) {
    // logic in threads
    let t0 = Instant::now();
    let keys = population.creatures.keys().collect::<Vec<&usize>>();
    let mut new_creatures = Vec::<Creature>::new();
    crossbeam::scope(|s| {
        let (tx, rx) = channel::<Creature>();
        let world = &world;
        let population = &population;
        let mut threads_num = 0;
        for keys_chunk in keys.chunks(pop_chunk_size) {
            threads_num += 1;
            let tx = tx.clone();
            s.spawn(move |_| {
                keys_chunk
                    .iter()
                    .for_each(|key| creature_logic(world, population, *key, &tx));
            });
        }
        println!("Spawned threads: {}", threads_num);

        drop(tx);
        while let Ok(creature) = rx.recv() {
            new_creatures.push(creature);
        }
    })
    .unwrap();
    let threads_eps = t0.elapsed().as_secs_f64();
    println!("Thread time: {:.3} sec", threads_eps);

    // remove creatures if to_remove == true
    let t0 = Instant::now();
    let mut num: usize = 0;
    population
        .creatures
        .retain(|k, v| {
            if !v.get_mut().unwrap().to_remove {
                true
            } else {
                num += 1;
                free_creature_keys.push(*k);
                false
            }
        });
    let remove_eps = t0.elapsed().as_secs_f64();
    println!("Removing {} creatures: {:.3} sec", num, remove_eps);

    // place new creatures
    let t0 = Instant::now();
    //println!("===Population: {}, free_creature_keys: {}, new_creatures: {}", population.creatures.len(), free_creature_keys.len(), new_creatures.len());
    while !new_creatures.is_empty() {
        let c_opt = new_creatures.pop();
        match c_opt {
            Some(c) => {
                let key_opt = free_creature_keys.pop();
                let key: usize;
                match key_opt {
                    Some(value) => key = value,
                    None => key = population.creatures.len(),
                }
                let a = world.areas.get_mut(&c.key_area).unwrap().get_mut().unwrap(); // tudo: try to par this
                a.key_creature = Some(key);
                assert!(!a.vacant, "vacant must be already false");
                let insert_opt = population.creatures.insert(key, RwLock::new(c));
                assert!(insert_opt.is_none(), "key {} already exists", key);
            },
            None => panic!("pop from empty new_creatures: impossible while !new_creatures.is_empty()"),
        }
    }
    //println!("===Population: {}, free_creature_keys: {}, new_creatures: {}", population.creatures.len(), free_creature_keys.len(), new_creatures.len());
    let create_eps = t0.elapsed().as_secs_f64();
    println!("Place new creatures {:.3} sec", create_eps);
}

fn structure_test(world: &World, population: &Population, verbose: bool) {
    print!("Structure test... ");
    if verbose {
        println!();
        world
            .areas
            .iter()
            .for_each(|(k, v)| println!("Area key: {:?}, {:?}", k, v));
        population
            .creatures
            .iter()
            .for_each(|(k, v)| println!("Creature key: {}, {:?}", k, v));
        println!("World size: {:?}", world.areas.len());
        println!("Population size: {:?}", population.creatures.len());
    }

    population.creatures.iter().for_each(|(key_creature, v)| {
        let m = v.read().unwrap();
        let area_opt = world.areas.get(&m.key_area);
        match area_opt {
            None => panic!("Creature contains incorrect key_area"),
            Some(area_mtx) => {
                let w = area_mtx.read().unwrap();
                assert!(
                    w.key_creature.is_some(),
                    "creature {} linked to area which does not have correct key_creature",
                    key_creature
                );
                assert!(
                    *key_creature == w.key_creature.unwrap(),
                    "capibara's key does not match corresponding area's key"
                );
            }
        }
        if m.to_remove {
            panic!("creature {} .to_remove must be false", key_creature);
        }
    });
    let mut i = 0;
    world.areas.iter().for_each(|(key_area, v)| {
        let w = v.read().unwrap();
        if w.key_creature.is_some() {
            i += 1;
            let m = population
                .creatures
                .get(&w.key_creature.unwrap())
                .unwrap()
                .read()
                .unwrap();
            assert!(
                *key_area == m.key_area,
                "area's key does not match corresponding capibara's key_area"
            );
            assert!(w.vacant == false, "vacant must be false");
        } else {
            assert!(w.vacant == true, "vacant must be true");
        }
    });
    assert!(
        i == population.creatures.len(),
        "areas containing creature_key: {}, population len {}",
        i,
        population.creatures.len()
    );

    println!("OK");
}

fn get_world_size(world: &World) -> (usize, usize) {
    (
        (mem::size_of::<RwLock<Area>>() + mem::size_of::<usize>()) * world.areas.len()
            + mem::size_of::<World>(),
        (mem::size_of::<RwLock<Area>>() + mem::size_of::<usize>()) * world.areas.capacity()
            + mem::size_of::<World>(),
    )
}

fn get_pop_size(pop: &Population) -> (usize, usize) {
    (
        (mem::size_of::<RwLock<Creature>>() + mem::size_of::<usize>()) * pop.creatures.len()
            + mem::size_of::<Population>(),
        (mem::size_of::<RwLock<Creature>>() + mem::size_of::<usize>()) * pop.creatures.capacity()
            + mem::size_of::<Population>(),
    )
}

fn main() {
    let world_size_x: usize = 2000;
    let world_size_y: usize = 2000;

    let pop_size = 1_000_000;
    let pop_chunk_size: usize = 50_000;

    let world_size = world_size_x * world_size_y;
    let mut world = World::new(TAreas::default());
    let mut population = Population::new(TCreatures::default());

    let mut free_creature_keys = Vec::<usize>::new();

    fill_world_population(
        &mut world,
        &mut population,
        (world_size_x, world_size_y),
        pop_size,
    );
    structure_test(&world, &population, false);
    println!();

    let (world_len, world_capacity) = get_world_size(&world);
    println!(
        "World size: {:.2} MB ({:.2} MB)",
        world_len as f64 / 1e6,
        world_capacity as f64 / 1e6
    );
    let (pop_len, pop_capacity) = get_pop_size(&population);
    println!(
        "Population size: {:.2} MB ({:.2} MB), {} creatures",
        pop_len as f64 / 1e6,
        pop_capacity as f64 / 1e6,
        population.creatures.len()
    );
    println!();

    threads_processing(&mut world, &mut population, &mut free_creature_keys, pop_chunk_size);
    println!();

    structure_test(&world, &population, false);
    println!();

    let (world_len, world_capacity) = get_world_size(&world);
    println!(
        "World size: {:.2} MB ({:.2} MB)",
        world_len as f64 / 1e6,
        world_capacity as f64 / 1e6
    );
    let (pop_len, pop_capacity) = get_pop_size(&population);
    println!(
        "Population size: {:.2} MB ({:.2} MB), {} creatures",
        pop_len as f64 / 1e6,
        pop_capacity as f64 / 1e6,
        population.creatures.len()
    );
}

fn creature_logic(world: &World, population: &Population, key: &usize, tx: &Sender<Creature>) {
    remove_creature(world, population, key);
    move_creature(world, population, key);
    new_creature(world, population, key, tx);
}

fn move_creature(world: &World, population: &Population, key: &usize) {
    let (x, y): (usize, usize);
    {
        let m = population.creatures.get(key).unwrap().read().unwrap();
        if m.to_remove {
            return;
        }
        (x, y) = m.key_area;
    }
    // Get free areas nearly
    let mut move_options: Vec<(usize, usize)> = Vec::new();
    move_options.push((x, y));

    for i in if x > 0 { x - 1 } else { 0 }..if x < world.size_x - 1 { x + 1 } else { x } + 1 {
        if i != x {
            move_options.push((i, y));
        }
    }
    for j in if y > 0 { y - 1 } else { 0 }..if y < world.size_y - 1 { y + 1 } else { y } + 1 {
        if j != y {
            move_options.push((x, j));
        }
    }

    let mut rng = rand::thread_rng();
    move_options.shuffle(&mut rng);

    let mut move_flag = false;
    for i in 0..move_options.len() {
        if move_options[i] == (x, y) {
            break;
        }
        {
            let mut a = world.areas.get(&move_options[i]).unwrap().write().unwrap();
            if a.vacant {
                a.key_creature = Some(*key);
                a.vacant = false;
                move_flag = true;
            }
        }
        if move_flag {
            {
                let mut m = population.creatures.get(key).unwrap().write().unwrap();
                m.key_area = move_options[i];
            }
            {
                let mut a = world.areas.get(&(x, y)).unwrap().write().unwrap();
                a.key_creature = None;
                a.vacant = true;
            }
            break;
        }
    }
}

fn remove_creature(world: &World, population: &Population, key: &usize) {
    // Remove creature with probability 10%
    if rand::random::<f32>() < 0.1 {
        let key_area: (usize, usize);
        {
            let mut m = population.creatures.get(key).unwrap().write().unwrap();
            if m.to_remove {
                return;
            }
            m.to_remove = true;
            key_area = m.key_area;
        }
        let mut a = world.areas.get(&key_area).unwrap().write().unwrap();
        a.key_creature = None;
        a.vacant = true;
    }
}

fn new_creature(world: &World, population: &Population, key: &usize, tx: &Sender<Creature>) {
    // if 2 nearest areas have another creature, key_creature.is_none(): new creature with proba = 0.5
    let (x, y): (usize, usize);
    {
        let m = population.creatures.get(key).unwrap().read().unwrap();
        if m.to_remove {
            return;
        }
        (x, y) = m.key_area;
    }

    // Get free areas nearly
    let mut nearest_keys_area: Vec<(usize, usize)> = Vec::new();
    for i in if x > 0 { x - 1 } else { 0 }..if x < world.size_x - 1 { x + 1 } else { x } + 1 {
        if i != x {
            nearest_keys_area.push((i, y));
        }
    }
    for j in if y > 0 { y - 1 } else { 0 }..if y < world.size_y - 1 { y + 1 } else { y } + 1 {
        if j != y {
            nearest_keys_area.push((x, j));
        }
    }

    let mut neib: Vec<usize> = Vec::new();
    let mut empty_areas: Vec<&(usize, usize)> = Vec::new();
    nearest_keys_area.iter().for_each(|key_area| {
        let a = world.areas.get(key_area).unwrap().read().unwrap();
        if a.vacant {
            empty_areas.push(key_area);
        } else if a.key_creature.is_some()  {
            neib.push(a.key_creature.unwrap());
        }
    });

    if neib.len() > 0 && empty_areas.len() > 0 {
        let mut rng = rand::thread_rng();
        empty_areas.shuffle(&mut rng);
        for key_area in empty_areas {       
            let mut a = world.areas.get(key_area).unwrap().write().unwrap();
            if a.vacant && rng.gen_range(0.0..=1.0) < 0.1 {
                a.vacant = false;
                tx.send(Creature::new(*key_area)).unwrap();
                return;
            }
        }
    }
}
