#![allow(unused_variables)]
#![allow(dead_code)]


use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam;
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use nohash_hasher::BuildNoHashHasher;
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
        a: usize,
        b: usize,
        to_remove: bool,
    }

    impl Capybara {
        pub fn new(param: usize) -> Capybara {
            Capybara { a: param, b: param, to_remove: false }
        }
    }

    // #[derive(Debug)]
    // struct Area {
    //     p: usize,
    // }

    // impl Area {
    //     pub fn new(p: usize) -> Area {
    //         Area { p }
    //     }
    // }

    // struct World {
    //     layout: IndexMap<usize, Mutex<Area>, BuildNoHashHasher<usize>>,
    // }

    // impl World {
    //     pub fn new(size: usize) -> Arc<World> {
    //         Arc::new(World {
    //             layout: IndexMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
    //         })
    //     }
    // }

    type Population = IndexMap<usize, Mutex<Capybara>, BuildNoHashHasher<usize>>;

    let mut population: Population = IndexMap::with_hasher(BuildNoHashHasher::default());
    for i in 0..10usize {
        population.insert(i, Mutex::new(Capybara::new(i)));
    }

    let keys = population.keys().collect::<Vec<&usize>>();
    

    let t0 = Instant::now();
    crossbeam::scope(|s| {
        let population = &population;
        
        for keys_chunk in keys.chunks(3) {
            s.spawn(move |_| {
                let thread_id = format!("{:?}", thread::current().id());
                let num = rand::thread_rng().gen_range(0..500);
                thread::sleep(Duration::from_millis(num));
                
                let k = *keys_chunk.first().unwrap();
                
                {   // mutate any capybara (first from the key_chunk)
                    let mut m = population.get(k).unwrap().lock().unwrap();
                    m.a = rand::thread_rng().gen_range(0..500);
                }
                println!("{:?}, sleep: {}, key: {}, {:?}", thread_id, num, k, population.get(k).unwrap());
                
                {   // mark any capybara (first from the key_chunk) to remove
                    let mut m = population.get(k).unwrap().lock().unwrap();
                    m.to_remove = true;
                }
                
            });
        }
    })
    .unwrap();
    let elapsed = t0.elapsed().as_secs_f64();
    println!("Time elapsed: {:.3} sec", elapsed);

    // remove capybaras if to_remove == true   retain, keys, par_keys
    population.retain(|_, m| !m.get_mut().unwrap().to_remove);


    // let mut world = World::new(size);
    // for i in 0_usize..size {
    //     world.layout.insert(i, Mutex::new(Area::new(i)));
    // }

    println!(
        "\nPopulation of {}, first: {:?}",
        population.len(),
        population[0]
    );

    // let selected_cells: Vec<&mut Mutex<Area>> = world
    // .layout
    // .values_mut()
    // .filter(|cell| {
    //     let m = cell.lock().unwrap();
    //     m.p >= 10 && m.p < 15
    // })
    // .collect();

    // crossbeam::scope(|s| {
    //     for chunk in selected_cells.chunks(2){

    //         s.spawn(move |_| {

    //             {   // mutate "own" cell from chunk
    //                 let mut m = chunk[0].lock().unwrap();
    //                 m.p = 222;
    //             }
    //             println!("Chunk {:?}", chunk);

    //             {   // mutate any cell from the world

    //                 //let a = world.map.first().unwrap().1;

    //             }
    //         });
    //     }

    // })
    // .unwrap();

    // println!("\nSelected cells: {:?}", selected_cells[0]);
    // println!("World: {:?}", world.layout.get_index(10));
}

fn main() {
    let size: usize = 20;
    let tests_num = 5;

    println!("HashMap size: {}", size);

    test2(size, tests_num);
}
