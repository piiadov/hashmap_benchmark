#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::Mutex;
use std::time::Instant;

use crossbeam;
use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use rand::seq::SliceRandom;
use rand::Rng;
use std::{mem, thread};

#[derive(Debug)]
struct Capybara {
    key_area: usize,
    to_remove: bool,
    to_move: Option<usize>,
    param: usize,
}

impl Capybara {
    pub fn new(key_area: usize) -> Capybara {
        Capybara {
            key_area,
            to_remove: false,
            to_move: None,
            param: 0,
        }
    }
}

#[derive(Debug)]
struct Area {
    key_capybara: Option<usize>,
    vacant: bool,
    param: usize,
    smth: f64,
}

impl Area {
    pub fn new() -> Area {
        Area {
            key_capybara: None,
            vacant: true,
            param: 0,
            smth: 123.123,
        }
    }
}

type World = IndexMap<usize, Mutex<Area>, BuildNoHashHasher<usize>>;
type Population = IndexMap<usize, Mutex<Capybara>, BuildNoHashHasher<usize>>;

fn fill_world_population(
    world: &mut World,
    population: &mut Population,
    world_size: usize,
    pop_size: usize,
) {
    // Fill world with areas
    print!("Create areas: ");
    let t0 = Instant::now();
    for i in 0..world_size {
        world.insert(i, Mutex::new(Area::new()));
    }
    println!("{:.3} sec", t0.elapsed().as_secs_f64());

    // Create capybaras in areas
    print!("Create population: ");
    let t0 = Instant::now();
    let mut rng = rand::thread_rng();
    let mut world_keys = world.keys().copied().collect::<Vec<usize>>();
    world_keys.shuffle(&mut rng);
    world_keys.truncate(pop_size);
    let mut irange = 0..world_keys.len();
    for k in world_keys {
        let i = irange.next().unwrap();
        let a = world.get_mut(&k).unwrap().get_mut().unwrap();
        let m_opt = population.insert(i, Mutex::from(Capybara::new(k)));
        match m_opt {
            Some(c) => panic!("Error: Cannot area already contains capybara {:?}", c),
            None => {
                a.key_capybara = Some(i);
                a.vacant = false;
            }
        }
    }
    println!("{:.3} sec", t0.elapsed().as_secs_f64());
}

fn threads_processing(world: &mut World, population: &mut Population, pop_chunk_size: usize) {
    // logic in threads
    let t0 = Instant::now();
    let keys = population.keys().collect::<Vec<&usize>>();
    crossbeam::scope(|s| {
        let world = &world;
        let population = &population;
        for keys_chunk in keys.chunks(pop_chunk_size) {
            s.spawn(move |_| {
                keys_chunk.iter().for_each(|key| capybara_logic(world, population, *key));
            });
        }
    })
    .unwrap();
    let thread_eps = t0.elapsed().as_secs_f64();
    println!("Thread time: {:.3} sec", thread_eps);

    // move capybaras if to_move == Some(key_area)
    print!("Moving capybaras: ");
    let t0 = Instant::now();
    population.iter_mut().for_each(|(key, v)| {
        let capybara = v.get_mut().unwrap();
        if capybara.to_move.is_some() {
            let current_area = world
                .get_mut(&capybara.key_area)
                .unwrap()
                .get_mut()
                .unwrap();
            current_area.key_capybara = None;
            current_area.vacant = true;

            let key_target_area = capybara.to_move.unwrap();
            capybara.key_area = key_target_area;
            capybara.to_move = None;

            let target_area = world.get_mut(&key_target_area).unwrap().get_mut().unwrap();
            target_area.key_capybara = Some(*key);
        }
    });
    let move_eps = t0.elapsed().as_secs_f64();
    println!("{:.3} sec", move_eps);

    // remove capybaras if to_remove == true
    print!("Removing capybaras: ");
    let t0 = Instant::now();
    population.retain(|_, v| {
        // clean from corresponding area
        let capybara = v.get_mut().unwrap();
        if capybara.to_remove {
            let area = world
                .get_mut(&capybara.key_area)
                .unwrap()
                .get_mut()
                .unwrap();
            area.key_capybara = None;
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
            .iter()
            .for_each(|(k, v)| println!("Area key: {}, {:?}", k, v));
        population
            .iter()
            .for_each(|(k, v)| println!("Capybara key: {}, {:?}", k, v));
        println!("World size: {:?}", world.len());
        println!("Population size: {:?}", population.len());
    }

    population.iter().for_each(|(key_capybara, v)| {
        let m = v.lock().unwrap();
        let area_opt = world.get(&m.key_area);
        match area_opt {
            None => panic!("Capybara contains incorrect key_area"),
            Some(area_mtx) => {
                let w = area_mtx.lock().unwrap();
                assert!(
                    w.key_capybara.is_some(),
                    "capybara {} linked to area which does not have correct key_capybara",
                    key_capybara
                );
                assert!(
                    *key_capybara == w.key_capybara.unwrap(),
                    "capibara's key does not match corresponding area's key"
                );
            }
        }
        if m.to_remove {
            panic!("capybara {} .to_remove must be false", key_capybara);
        }
        if m.to_move.is_some() {
            panic!("capybara {} .to_move must be None", key_capybara);
        }
    });
    let mut i = 0;
    world.iter().for_each(|(key_area, v)| {
        let w = v.lock().unwrap();
        if w.key_capybara.is_some() {
            i += 1;
            let m = population
                .get(&w.key_capybara.unwrap())
                .unwrap()
                .lock()
                .unwrap();
            assert!(
                *key_area == m.key_area,
                "area's key does not match corresponding capibara's key_area"
            );
        }
        if w.vacant {
            assert!(
                w.key_capybara.is_none(),
                "area {} is vacant but has capybara {}",
                key_area,
                w.key_capybara.unwrap()
            );
        } else {
            assert!(
                w.key_capybara.is_some(),
                "area {} is not vacant and does not have capybara",
                key_area
            );
        }
    });
    assert!(
        i == population.len(),
        "areas containing capybara_key: {}, population len {}",
        i,
        population.len()
    );

    println!("OK");
}

fn get_world_size(world: &World) -> (usize, usize) {
    (
        (mem::size_of::<Mutex<Area>>() + mem::size_of::<usize>()) * world.len()
            + mem::size_of::<World>(),
        (mem::size_of::<Mutex<Area>>() + mem::size_of::<usize>()) * world.capacity()
            + mem::size_of::<World>(),
    )
}

fn get_pop_size(pop: &Population) -> (usize, usize) {
    (
        (mem::size_of::<Mutex<Capybara>>() + mem::size_of::<usize>()) * pop.len()
            + mem::size_of::<Population>(),
        (mem::size_of::<Mutex<Capybara>>() + mem::size_of::<usize>()) * pop.capacity()
            + mem::size_of::<Population>(),
    )
}

fn capybara_logic_example(world: &World, population: &Population, keys: &[&usize]) {
    let k = *keys.first().unwrap();
    let kk = *keys.last().unwrap();

    {
        // Mutate any capybara (first from the key_chunk)
        let mut m = population.get(k).unwrap().lock().unwrap();
        m.param = rand::thread_rng().gen_range(0..500);
    }

    {
        // Mark any capybara (first from the key_chunk) to remove
        let mut m = population.get(k).unwrap().lock().unwrap();
        m.to_remove = true;
    }

    {
        // Mutate area
        let m = population.get(k).unwrap().lock().unwrap();
        let mut w = world.get(&m.key_area).unwrap().lock().unwrap();
        w.param = 555;
    }

    {
        // Mark capybara to move
        let key_target_area = rand::thread_rng().gen_range(0..world.len());
        if *kk != key_target_area {
            let mut w = world.get(&key_target_area).unwrap().lock().unwrap();
            if w.vacant == true {
                let mut m = population.get(kk).unwrap().lock().unwrap();
                w.vacant = false;
                m.to_move = Some(key_target_area);
            }
        }
    }
}

fn main() {
    let world_size_x: usize = 16;
    let world_size_y: usize = 16;

    let pop_size = 5;//_000_000;
    let pop_chunk_size: usize = 2;//_000_000;

    let world_size = world_size_x * world_size_y;
    let mut world: World =
        IndexMap::with_capacity_and_hasher(world_size, BuildNoHashHasher::default());
    let mut population: Population = IndexMap::with_hasher(BuildNoHashHasher::default());

    fill_world_population(&mut world, &mut population, world_size, pop_size);
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

fn capybara_logic(world: &World, population: &Population, key: &usize) {
    // move
    let mut m = population.get(key).unwrap().lock().unwrap();

    dbg!(m);


}

