
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::time::Instant;


struct Cell {
    p: usize
}

impl Cell {
    pub fn new(p: usize) -> Cell{
        Cell { p }
    }
}


fn main() {
    let size: usize = 10_000_000;
    let tests_num = 5;

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
            let mut sum: usize = 0;
            for i in 0_usize..size {
                if let Some(x) = hm.get(&i) {
                    sum += x.p as usize;
                }
            }
            let elapsed = t0.elapsed().as_secs_f64();
            println!("The sum is: {}. Time elapsed: {:.3} sec", sum, elapsed);
        }
    }


}

