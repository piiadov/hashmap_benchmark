#![allow(unused_variables)]
#![allow(dead_code)]

//use rayon::prelude::*;
use indexmap::IndexMap;

use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::time::Instant;


#[derive(Debug)]
struct Capybara {
    a: usize,
    b: usize,
}

impl Capybara {
    pub fn new(a: usize) -> Option<Capybara> {
        Some(Capybara {a, b:a})
        //None
    }
}

#[derive(Debug)]
struct Cell {
    p: usize,
    capybara: Option<Capybara>,
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
    map: IndexMap::<usize, Cell, BuildNoHashHasher<usize>>,
}

impl CellsMap {
    pub fn new(size: usize) -> CellsMap {
        CellsMap {
            map: IndexMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
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

    let mut selected_cells: Vec<(&usize, &mut Cell)> = cmap.map.iter_mut().filter(|cell| {
        cell.1.p >=1 && cell.1.p < 10
    }).collect();

    let elapsed = t0.elapsed().as_secs_f64();
    println!("Time elapsed: {:.3} sec", elapsed);

    // Check changes in &Capybara borrowing capybara from Cell
    let num = 3usize;

    println!("{:?}", &selected_cells[num]);

    //let cap = cmap.map.capacity();
    //println!("Capacity: {:?}", cmap.map.capacity());

    let c_opt = selected_cells[num].1.capybara.as_mut();
    match c_opt {
        None => println!("No capybara in cell {}", num),
        Some(c) => {
            println!("Capybara in selected cell {}: {:?}", num, c);
            // change in capybara
            c.a = 555;
            println!("Capybara changed: {:?}", c);
        },
    }
    
    // Check if capybara keep changes
    let c_opt2 = selected_cells[num].1.capybara.as_mut();
    println!("Capybara keep the change: {:?}", c_opt2);
    
    // Check changes in a Cell from CellsMap
    println!("{:?}", selected_cells[num].1);
    selected_cells[num].1.p = 123;
    println!("{:?}", selected_cells[num].1);

    println!("CellsMap capacity: {:?}", &cmap.map.capacity());

    // Mutable slices
    println!("\nMutable slices:\n");

    let slice_opt = cmap.map.get_range_mut(0..2);
    match slice_opt {
        None => println!("Something went wrong with get_range_mut on CellsMap"),
        Some(slice) => {

            println!("{:?}", slice[0]);
            slice[0].p = 11;
            println!("{:?}", slice[0]);

            let a = & slice[0];
            let b = & slice[1];
            
            println!("{:?}", a);

        },
    }








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
    let size: usize = 15;
    let tests_num = 5;

    println!("HashMap size: {}", size);
    
    test1(size, tests_num);
    test2(size, tests_num);

}
