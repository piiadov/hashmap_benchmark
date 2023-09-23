#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crossbeam;
use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use rand::seq::SliceRandom;
use rand::Rng;
use std::thread;

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
}

impl Area {
    pub fn new() -> Area {
        Area {
            key_capybara: None,
            vacant: true,
            param: 0,
        }
    }
}

type World = IndexMap<usize, Mutex<Area>, BuildNoHashHasher<usize>>;
type Population = IndexMap<usize, Mutex<Capybara>, BuildNoHashHasher<usize>>;

fn patterns() -> (World, Population) {
    println!("\nThreads access to random cell:\n");

    let mut world: World = IndexMap::with_hasher(BuildNoHashHasher::default());
    let mut population: Population = IndexMap::with_hasher(BuildNoHashHasher::default());

    let world_size = 20usize;
    let pop_size = 10usize;

    // Fill world with areas
    for i in 0..world_size {
        world.insert(i, Mutex::new(Area::new()));
    }

    // Create capybaras in areas
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

    world
        .iter()
        .for_each(|(k, v)| println!("Area key: {}, {:?}", k, v));
    population
        .iter()
        .for_each(|(k, v)| println!("Capybara key: {}, {:?}", k, v));
    println!("World size: {:?}", world.len());
    println!("Population size: {:?}", population.len());

    // Process capybaras in threads
    println!("\nThreads start:");
    let t0 = Instant::now();
    let keys = population.keys().collect::<Vec<&usize>>();
    crossbeam::scope(|s| {
        let world = &world;
        let population = &population;
        for keys_chunk in keys.chunks(3) {
            s.spawn(move |_| {
                let thread_id = format!("{:?}", thread::current().id());
                let num = rand::thread_rng().gen_range(0..500);
                thread::sleep(Duration::from_millis(num));

                let k = *keys_chunk.first().unwrap();
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
                    let k = *keys_chunk.last().unwrap();
                    if *k != key_target_area {
                        let mut w = world.get(&key_target_area).unwrap().lock().unwrap();
                        if w.vacant == true {
                            let mut m = population.get(k).unwrap().lock().unwrap();
                            w.vacant = false;
                            m.to_move = Some(key_target_area);
                        }
                    }
                }

                println!(
                    "{:?}, sleep: {}, key: {}, {:?}",
                    thread_id,
                    num,
                    k,
                    population.get(k).unwrap()
                );
            });
        }
    })
    .unwrap();
    let elapsed = t0.elapsed().as_secs_f64();
    println!("Time elapsed: {:.3} sec", elapsed);

    // move capybaras if to_move == Some(key_area)
    population.iter_mut().for_each(|(_, v)| {
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
        }
    });

    // remove capybaras if to_remove == true
    population.retain(|_, v| {
        // clean from corresponding area
        let capybara = v.get_mut().unwrap();
        if capybara.to_remove {
            world
                .get_mut(&capybara.key_area)
                .unwrap()
                .get_mut()
                .unwrap()
                .key_capybara = None;
            false
        } else {
            true
        }
    });

    println!(
        "\nPopulation of {}, first: {:?}",
        population.len(),
        population[0]
    );

    world
        .iter()
        .for_each(|(k, v)| println!("Area key: {}, {:?}", k, v));
    population
        .iter()
        .for_each(|(k, v)| println!("Capybara key: {}, {:?}", k, v));
    println!("World size: {:?}", world.len());
    println!("Population size: {:?}", population.len());

    (world, population)
}

fn main() {
    let (world, population) = patterns();
}
