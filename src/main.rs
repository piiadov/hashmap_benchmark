#![allow(unused_variables)]
#![allow(dead_code)]

//use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug)]
struct Capybara {
    a: usize,
    b: usize,
}

impl Capybara {
    pub fn new(a: usize) -> Capybara {
        Capybara {
            a , 
            b:a
        }
    }
}

#[derive(Debug)]
struct Cell {
    p: usize,
    capybara: Capybara,
}

impl Cell {
    pub fn new(p: usize) -> Cell{
        Cell { 
            p,
            capybara: Capybara::new(p),
        }
    }
}


struct CellsMap {
    map: HashMap::<usize, Cell, BuildNoHashHasher<usize>>,
}

impl CellsMap {
    pub fn new(size: usize) -> CellsMap {
        CellsMap {
            map: HashMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
        }
    }
}

fn test2(size: usize, tests_num: usize) {
    println!("\n===== TEST 2 =====");
    println!("----- Write CellsMap performance-----");
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
    let selected_cells: Vec<_> = cmap.map.iter().filter(|cell| {
        cell.1.p > 30000 && cell.1.p < 100000
    }).collect();
    let elapsed = t0.elapsed().as_secs_f64();
    println!("Time elapsed: {:.3} sec", elapsed);
    println!("{:?}", &selected_cells[100]);
}

fn test1(size: usize, tests_num: usize) {
    println!("\n===== TEST 1 =====");
    //let mut hm: HashMap::<usize, Cell> = HashMap::new();
    //let mut hm: HashMap::<usize, Cell, BuildNoHashHasher<usize>> = HashMap::with_capacity_and_hasher(size, BuildNoHashHasher::default());
    let mut hm: HashMap::<usize, Cell, BuildNoHashHasher<usize>> = HashMap::with_hasher(BuildNoHashHasher::default());
    {    
        println!("----- Write HashMap/BuildNoHashHasher (nohash-hasher 0.2.0) -----");
        for _ in 0..tests_num {
            let t0 = Instant::now();
            for i in 0_usize..size {                
                hm.insert(i, Cell::new(i));
            }
            let elapsed = t0.elapsed().as_secs_f64();
            println!("Time elapsed: {:.3} sec", elapsed);
        }
    }
    {
        println!("----- Read HashMap/BuildNoHashHasher (nohash-hasher 0.2.0) -----");
        for _ in 0..tests_num {
            let t0 = Instant::now();
            let mut sum: f32 = 0f32;
            for i in 0_usize..size {
                if let Some(x) = hm.get(&i) {
                    sum += x.p as f32;
                }
            }
            let elapsed = t0.elapsed().as_secs_f64();
            println!("The sum is: {}. Time elapsed: {:.3} sec", sum, elapsed);
        }
    }
}

fn main() {
    let size: usize = 100_000_000;
    let tests_num = 5;

    println!("HashMap size: {}", size);
    
    test1(size, tests_num);
    test2(size, tests_num);

}
