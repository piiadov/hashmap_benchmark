#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam;
use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use rand::seq::SliceRandom;
use rand::Rng;
use std::thread;

fn test2(size: usize, tests_num: usize) {
    {
        println!("\n===== TEST 2 =====");
        println!("----- Write CellsMap performance-----");

        #[derive(Debug)]
        struct Capybara {
            a: usize,
            b: usize,
        }

        impl Capybara {
            pub fn new(a: usize) -> Option<Capybara> {
                Some(Capybara { a, b: a })
                //None
            }
        }

        #[derive(Debug)]
        struct Cell {
            p: usize,
            capybara: Option<Capybara>,
        }

        impl Cell {
            pub fn new(p: usize) -> Cell {
                Cell {
                    p,
                    capybara: Capybara::new(p),
                }
            }
        }

        struct CellsMap {
            map: IndexMap<usize, Cell, BuildNoHashHasher<usize>>,
        }

        impl CellsMap {
            pub fn new(size: usize) -> CellsMap {
                CellsMap {
                    map: IndexMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
                }
            }
        }

        for _ in 0..tests_num {
            let mut cmap = CellsMap::new(size);
            let t0 = Instant::now();
            for i in 0_usize..size {
                cmap.map.insert(i, Cell::new(i));
            }
            let elapsed = t0.elapsed().as_secs_f64();
            println!("Time elapsed: {:.3} sec", elapsed);
        }

        println!("----- Lookup performance -----");

        let mut cmap = CellsMap::new(size);
        for i in 0_usize..size {
            cmap.map.insert(i, Cell::new(i));
        }

        // lookup cells

        let t0 = Instant::now();

        let mut selected_cells: Vec<(&usize, &mut Cell)> = cmap
            .map
            .iter_mut()
            .filter(|cell| cell.1.p >= 1 && cell.1.p < 10)
            .collect();

        let elapsed = t0.elapsed().as_secs_f64();
        println!("Time elapsed: {:.3} sec", elapsed);

        // Check changes in &Capybara borrowing capybara from Cell
        let num = 3usize;

        println!("{:?}", &selected_cells[num]);

        //println!("Capacity: {:?}", cmap.map.capacity());

        let c_opt = selected_cells[num].1.capybara.as_mut();
        match c_opt {
            None => println!("No capybara in cell {}", num),
            Some(c) => {
                println!("Capybara in selected cell {}: {:?}", num, c);
                // change in capybara
                c.a = 555;
                println!("Capybara changed: {:?}", c);
            }
        }

        // Check if capybara keep changes
        let c_opt2 = selected_cells[num].1.capybara.as_mut();
        println!("Capybara keep the change: {:?}", c_opt2);

        // Check changes in a Cell from CellsMap
        println!("{:?}", selected_cells[num].1);
        selected_cells[num].1.p = 123;
        println!("{:?}", selected_cells[num].1);

        println!("CellsMap capacity: {:?}", &cmap.map.capacity());
    }

    {
        // Mutable slices
        println!("\nMutable slices, keep owner:\n");
        #[derive(Debug)]
        struct Cell {
            p: usize,
            capybara: Option<Capybara>,
        }

        impl Cell {
            pub fn new(p: usize) -> Cell {
                Cell {
                    p,
                    capybara: Capybara::new(p),
                }
            }
        }

        struct CellsMap {
            map: IndexMap<usize, Cell, BuildNoHashHasher<usize>>,
        }

        impl CellsMap {
            pub fn new(size: usize) -> CellsMap {
                CellsMap {
                    map: IndexMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
                }
            }
        }
        #[derive(Debug)]
        struct Capybara {
            a: usize,
            b: usize,
        }

        impl Capybara {
            pub fn new(a: usize) -> Option<Capybara> {
                Some(Capybara { a, b: a })
                //None
            }
        }

        let mut cmap = CellsMap::new(size);
        for i in 0_usize..size {
            cmap.map.insert(i, Cell::new(i));
        }

        let mut selected_cells: Vec<&mut Cell> = cmap
            .map
            .values_mut()
            .filter(|cell| cell.p >= 10 && cell.p < 20)
            .collect();

        let r = &mut selected_cells as *mut Vec<&mut Cell>;

        unsafe {
            (&mut *r)[0].p = 222;
            (&mut *r)[0].capybara.as_mut().unwrap().a = 55;
        }

        unsafe {
            (&mut *r)[1].p = 333;
        }

        println!("Selected cells: {:?}", selected_cells[0]);
        println!("Selected cells: {:?}", selected_cells[1]);

        println!("CellsMap: {:?}", cmap.map.get_index(10)); // Finally, CellsMap data is not borrowed
    }

    {
        // Mutable cells in threads
        println!("\nThreads:\n");
        #[derive(Debug)]
        struct Cell {
            p: usize,
            capybara: Option<Capybara>,
        }

        impl Cell {
            pub fn new(p: usize) -> Cell {
                Cell {
                    p,
                    capybara: Capybara::new(p),
                }
            }
        }

        struct CellsMap {
            map: IndexMap<usize, Cell, BuildNoHashHasher<usize>>,
        }

        impl CellsMap {
            pub fn new(size: usize) -> CellsMap {
                CellsMap {
                    map: IndexMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
                }
            }
        }
        #[derive(Debug)]
        struct Capybara {
            a: usize,
            b: usize,
        }

        impl Capybara {
            pub fn new(a: usize) -> Option<Capybara> {
                Some(Capybara { a, b: a })
                //None
            }
        }

        let mut cmap = CellsMap::new(size);
        for i in 0_usize..size {
            cmap.map.insert(i, Cell::new(i));
        }

        let mut selected_cells: Vec<Arc<Mutex<&mut Cell>>> = cmap
            .map
            .values_mut()
            .filter(|cell| cell.p >= 10 && cell.p < 15)
            .map(|cell| Arc::new(Mutex::new(cell)))
            .collect();

        println!("Main thread id: {:?}", thread::current().id());
        crossbeam::scope(|s| {
            let mtx = &mut selected_cells[0];
            for i in 0..5 {
                let m = Arc::clone(&mtx);
                s.spawn(move |_| {
                    let num = rand::thread_rng().gen_range(10..50);
                    thread::sleep(Duration::from_millis(num));

                    let mut mlock = m.lock().unwrap();

                    mlock.p = rand::thread_rng().gen_range(100..500);
                    mlock.capybara.as_mut().unwrap().a = rand::thread_rng().gen_range(100..500);
                    println!("{:?}:  {:?}", thread::current().id(), mlock);
                });
            }
        })
        .unwrap();

        println!("Selected cells: {:?}", selected_cells[0]);
        println!("CellsMap: {:?}", cmap.map.get_index(10));
    }

    println!("\nThreads access to random cell:\n");

    #[derive(Debug)]
    struct Capybara {
        key: usize,
        key_area: usize,
        to_remove: bool,
        param: usize,
    }

    impl Capybara {
        pub fn new(key: usize, key_area: usize) -> Capybara {
            Capybara {
                key,
                key_area,
                to_remove: false,
                param: 0,
            }
        }
    }

    #[derive(Debug)]
    struct Area {
        key: usize,
        key_capybara: Option<usize>,
        param: usize,
    }

    impl Area {
        pub fn new(key: usize) -> Area {
            Area {
                key,
                key_capybara: None,
                param: 0,
            }
        }
    }

    type World = IndexMap<usize, Mutex<Area>, BuildNoHashHasher<usize>>;
    type Population = IndexMap<usize, Mutex<Capybara>, BuildNoHashHasher<usize>>;

    let mut world: World = IndexMap::with_hasher(BuildNoHashHasher::default());
    let mut population: Population = IndexMap::with_hasher(BuildNoHashHasher::default());

    let world_size = 20usize;
    let pop_size = 10usize;

    // Fill world with areas
    for i in 0..world_size {
        world.insert(i, Mutex::new(Area::new(i)));
    }

    // Create capybaras in areas
    let mut rng = rand::thread_rng();
    let mut world_keys = world.keys().copied().collect::<Vec<_>>();
    world_keys.shuffle(&mut rng);
    world_keys.truncate(pop_size);
    let mut irange = 0..world_keys.len();
    for k in world_keys {
        let i = irange.next().unwrap();
        let a = world.get_mut(&k).unwrap().get_mut().unwrap();
        let m_opt = population.insert(i, Mutex::from(Capybara::new(i, k)));
        match m_opt {
            Some(c) => panic!("Error: Cannot area already contains capybara {:?}", c),
            None => a.key_capybara = Some(i),
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
    // todo

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
}

fn main() {
    let size: usize = 20;
    let tests_num = 5;

    println!("HashMap size: {}", size);

    test2(size, tests_num);
}
