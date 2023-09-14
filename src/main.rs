
use nohash_hasher::BuildNoHashHasher;
use std::collections::{HashMap, HashSet};
use std::time::Instant;


struct Cell {
    p: usize,
}

impl Cell {
    pub fn new(p: usize) -> Cell{
        Cell { p }
    }
}


struct CellsMap {
    available_keys: HashSet<usize, BuildNoHashHasher<usize>>,
    map: HashMap::<usize, Cell, BuildNoHashHasher<usize>>,
}

impl CellsMap {
    pub fn new(size: usize) -> CellsMap {
        let mut available_keys = HashSet::with_capacity_and_hasher(size, BuildNoHashHasher::default());
        for i in 0_usize..size {
            available_keys.insert(i);
        }
        CellsMap {
            available_keys,
            map: HashMap::with_capacity_and_hasher(size, BuildNoHashHasher::default()),
        }
    }

    fn insert(&mut self, cell: Cell) {
        if self.available_keys.len() == 0 {
            panic!("(CellsMap::insert) CellsMap is full, cannot insert new cell");
        }
        let key = self.available_keys.iter().nth(0).copied().unwrap();
        self.map.insert(key, cell);
        
        let c = self.map.get_mut(&key).unwrap();
        c.p = 10;

        //self.map.remove(&key);

        // bottleneck on element remove
        //self.available_keys.remove(&key);
    }
}

fn test2(size: usize, tests_num: usize) {
    println!("\n===== TEST 2 =====");
    println!("----- Write Cellsmap -----");
    for _ in 0..tests_num {
        let mut cmap = CellsMap::new(size);
        let t0 = Instant::now();
        for i in 0_usize..size {                
            cmap.insert(Cell::new(i));
        }
        let elapsed = t0.elapsed().as_secs_f64();
        println!("Time elapsed: {:.3} sec", elapsed);
    }


}

// fn test1(size: usize, tests_num: usize) {
//     println!("\n===== TEST 1 =====");
//     //let mut hm: HashMap::<usize, Cell> = HashMap::new();
//     //let mut hm: HashMap::<usize, Cell, BuildNoHashHasher<usize>> = HashMap::with_capacity_and_hasher(size, BuildNoHashHasher::default());
//     let mut hm: HashMap::<usize, Cell, BuildNoHashHasher<usize>> = HashMap::with_hasher(BuildNoHashHasher::default());
//     {    
//         println!("----- Write HashMap/BuildNoHashHasher (nohash-hasher 0.2.0) -----");
//         for _ in 0..tests_num {
//             let t0 = Instant::now();
//             for i in 0_usize..size {                
//                 hm.insert(i, Cell::new(i));
//             }
//             let elapsed = t0.elapsed().as_secs_f64();
//             println!("Time elapsed: {:.3} sec", elapsed);
//         }
//     }
//     {
//         println!("----- Read HashMap/BuildNoHashHasher (nohash-hasher 0.2.0) -----");
//         for _ in 0..tests_num {
//             let t0 = Instant::now();
//             let mut sum: f32 = 0f32;
//             for i in 0_usize..size {
//                 if let Some(x) = hm.get(&i) {
//                     sum += x.p as f32;
//                 }
//             }
//             let elapsed = t0.elapsed().as_secs_f64();
//             println!("The sum is: {}. Time elapsed: {:.3} sec", sum, elapsed);
//         }
//     }
// }

fn main() {
    let size: usize = 1_000_0000;
    let tests_num = 5;

    println!("HashMap size: {}", size);
    
    //test1(size, tests_num);
    test2(size, tests_num);

}
