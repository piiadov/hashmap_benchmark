#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::time::Instant;
use std::{hash::BuildHasherDefault, sync::RwLock};

use crossbeam;
use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use rand::seq::SliceRandom;
use rand::Rng;
use seahash::SeaHasher;
use std::{default, mem, thread};

#[derive(Debug)]
struct Creature {
    key_area: (usize, usize),
    to_remove: bool,
    to_move: Option<(usize, usize)>,
    param: usize,
}

impl Creature {
    pub fn new(key_area: (usize, usize)) -> Creature {
        Creature {
            key_area,
            to_remove: false,
            to_move: None,
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

fn threads_processing(world: &mut World, population: &mut Population, pop_chunk_size: usize) {
    // logic in threads
    let t0 = Instant::now();
    let keys = population.creatures.keys().collect::<Vec<&usize>>();
    crossbeam::scope(|s| {
        let world = &world;
        let population = &population;
        let mut threads_num = 0;
        for keys_chunk in keys.chunks(pop_chunk_size) {
            threads_num += 1;
            s.spawn(move |_| {
                keys_chunk
                    .iter()
                    .for_each(|key| creature_logic(world, population, *key));
            });
        }
        println!("Spawned threads: {}", threads_num);
    })
    .unwrap();
    let thread_eps = t0.elapsed().as_secs_f64();
    println!("Thread time: {:.3} sec", thread_eps);

    // move creatures if to_move == Some(key_area)
    print!("Moving creatures: ");
    let t0 = Instant::now();
    population.creatures.iter_mut().for_each(|(key, v)| {
        let creature = v.get_mut().unwrap();
        if creature.to_move.is_some() {
            let current_area = world
                .areas
                .get_mut(&creature.key_area)
                .unwrap()
                .get_mut()
                .unwrap();
            current_area.key_creature = None;
            current_area.vacant = true;

            let key_target_area = creature.to_move.unwrap();
            creature.key_area = key_target_area;
            creature.to_move = None;

            let target_area = world
                .areas
                .get_mut(&key_target_area)
                .unwrap()
                .get_mut()
                .unwrap();
            target_area.key_creature = Some(*key);
        }
    });
    let move_eps = t0.elapsed().as_secs_f64();
    println!("{:.3} sec", move_eps);

    // remove creatures if to_remove == true
    print!("Removing creatures: ");
    let t0 = Instant::now();
    population.creatures.retain(|_, v| {
        // clean from corresponding area
        let creature = v.get_mut().unwrap();
        if creature.to_remove {
            let area = world
                .areas
                .get_mut(&creature.key_area)
                .unwrap()
                .get_mut()
                .unwrap();
            area.key_creature = None;
            area.vacant = true;
            false
        } else {
            true
        }
    });
    let remove_eps = t0.elapsed().as_secs_f64();
    println!("{:.3} sec", remove_eps);
    println!(
        "Time elapsed: {:.3} sec",
        thread_eps + move_eps + remove_eps
    );
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
        if m.to_move.is_some() {
            panic!("creature {} .to_move must be None", key_creature);
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
        }
        if w.vacant {
            assert!(
                w.key_creature.is_none(),
                "area {:?} is vacant but has creature {}",
                key_area,
                w.key_creature.unwrap()
            );
        } else {
            assert!(
                w.key_creature.is_some(),
                "area {:?} is not vacant and does not have creature",
                key_area
            );
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

// fn creature_logic_example(world: &World, population: &Population, keys: &[&usize]) {
//     let k = *keys.first().unwrap();
//     let kk = *keys.last().unwrap();

//     {
//         // Mutate any creature (first from the key_chunk)
//         let mut m = population.get(k).unwrap().write().unwrap();
//         m.param = rand::thread_rng().gen_range(0..500);
//     }

//     {
//         // Mark any creature (first from the key_chunk) to remove
//         let mut m = population.get(k).unwrap().write().unwrap();
//         m.to_remove = true;
//     }

//     {
//         // Mutate area
//         let m = population.get(k).unwrap().read().unwrap();
//         let mut w = world.get(&m.key_area).unwrap().write().unwrap();
//         w.param = 555;
//     }

//     {
//         // Mark creature to move
//         let key_target_area = rand::thread_rng().gen_range(0..world.len());
//         if *kk != key_target_area {
//             let mut w = world.get(&key_target_area).unwrap().write().unwrap();
//             if w.vacant == true {
//                 let mut m = population.get(kk).unwrap().write().unwrap();
//                 w.vacant = false;
//                 m.to_move = Some(key_target_area);
//             }
//         }
//     }
// }

fn main() {
    let world_size_x: usize = 3000;
    let world_size_y: usize = 3000;

    let pop_size = 5_000_000;
    let pop_chunk_size: usize = 500_000;

    let world_size = world_size_x * world_size_y;
    let mut world = World::new(TAreas::default());
    let mut population = Population::new(TCreatures::default());

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
        "Population size: {:.2} MB ({:.2} MB)",
        pop_len as f64 / 1e6,
        pop_capacity as f64 / 1e6
    );
    println!();

    threads_processing(&mut world, &mut population, pop_chunk_size);
    println!();

    println!("Check world after processing: ");
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
        "Population size: {:.2} MB ({:.2} MB)",
        pop_len as f64 / 1e6,
        pop_capacity as f64 / 1e6
    )
}

fn creature_logic(world: &World, population: &Population, key: &usize) {
    move_creature(world, population, key);

    
        
    // let key_target_area = (rand::thread_rng().gen_range(0..3000), rand::thread_rng().gen_range(0..3000));
    // let mut w = world.areas.get(&key_target_area).unwrap().write().unwrap();
    // if w.vacant == true {
    //     let mut m = population.creatures.get(key).unwrap().write().unwrap();
    //     w.vacant = false;
    //     m.to_move = Some(key_target_area);
    // }
        
    
}

fn move_creature(world: &World, population: &Population, key: &usize) {
    // Mark creature to move
    let (x, y): (usize, usize);
    {
        let m = population.creatures.get(key).unwrap().read().unwrap();
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

    let mut to_move: Option<(usize, usize)> = None;
    for i in 0..move_options.len() {
        if move_options[i] == (x, y) {
            break;
        }
        {
            let mut a = world.areas.get(&move_options[i]).unwrap().write().unwrap();
            if a.vacant {
                a.vacant = false;
                to_move = Some(move_options[i]);
            }
        }
        if to_move.is_some() {
            {
                let mut m = population.creatures.get(key).unwrap().write().unwrap();
                m.to_move = to_move;
            }
            break;
        }


    }
}
