#![allow(dead_code)]
extern crate rand;
extern crate mpc_framework;
use mpc_framework::*;
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use std::path::Path;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let dir = Path::new(&args[1]);
    let parameters = File::open(dir.join(Path::new(&args[2] as &str))).unwrap();
    let reader_parameters = BufReader::new(parameters);
    let mut this_party = 0;
    let mut num_inputs = 0;
    let mut k = 0;
    let mut threshold = 0.0;
    let mut max = 0;
    let mut decimal_points = 0;
    for l in reader_parameters.lines().take(6) {
        let split = l.unwrap().clone();
        let split = split.split(' ').collect::<Vec<_>>();
        match split[0] {
            "party" => this_party = (&split[1]).parse::<usize>().unwrap(),
            "n" => num_inputs = (&split[1]).parse::<u64>().unwrap(),
            "k" => k = (&split[1]).parse::<usize>().unwrap(),
            "max" => max = (&split[1]).parse::<uint_t>().unwrap(),
            "threshold" => threshold = (&split[1]).parse::<f64>().unwrap(),
            "decimal_points" => decimal_points = (&split[1]).parse::<usize>().unwrap(),
            _ => unreachable!(), 
        }
    }

    //gen_input(num_inputs as usize, max as u64, dir);
    //return;
    
    let input = File::open(&args[3] as &str).unwrap();
    let reader_input = BufReader::new(input);
    let mut bs = Vec::new();
    let mut cs = Vec::new();
    let mut id_map = Vec::new();

    for l in reader_input.lines().skip(1).take(num_inputs as usize) {
        let split = l.unwrap().clone();
        let split = split.split(' ').collect::<Vec<_>>();
        if split.len()!=3 { break; }
        id_map.push(split[0].to_string());
        bs.push((&split[1]).parse::<uint_t>().unwrap());
        cs.push((&split[2]).parse::<uint_t>().unwrap());
    }

    let shifts = (10 as usize).pow(decimal_points as u32).next_power_of_two()
        .trailing_zeros() as usize;
    let threshold = (threshold*((2 as int_t).pow(shifts as u32) as f64)) 
        as uint_t;

    let state = new_state(3, this_party, Vec::new(), Vec::new());

    let mut obliv_b = Vec::new();
    let mut obliv_c = Vec::new();
    for p in 0..this_party {
        obliv_b.push((0..(num_inputs as usize))
                     .map(|_| OblivUInt::new_from_max(None, max, p, 
                                                      state.clone()))
                     .collect::<Vec<_>>());
        obliv_c.push((0..(num_inputs as usize))
                     .map(|_| OblivUInt::new_from_max(None, max, p, 
                                                      state.clone()))
                     .collect::<Vec<_>>());

    }
    obliv_b.push(bs.clone().into_iter()
                 .map(|x| OblivUInt::new_from_max(Some(x), max, this_party, 
                                                  state.clone()))
                 .collect::<Vec<_>>());
    obliv_c.push(cs.clone().into_iter()
                 .map(|x| OblivUInt::new_from_max(Some(x), max, this_party, 
                                                  state.clone()))
                 .collect::<Vec<_>>());
    for p in (this_party+1)..3{
        obliv_b.push((0..(num_inputs as usize))
                     .map(|_| OblivUInt::new_from_max(None, max, p, 
                                                      state.clone()))
                     .collect::<Vec<_>>());
        obliv_c.push((0..(num_inputs as usize))
                     .map(|_| OblivUInt::new_from_max(None, max, p, 
                                                      state.clone()))
                     .collect::<Vec<_>>());
    }

    let obliv_bs = obliv_b[0].iter().zip(obliv_b[1].iter())
        .zip(obliv_b[2].iter())
        .map(|((b0, b1), b2)| b0.clone()+b1.clone()+b2.clone())
        .collect::<Vec<_>>();
    let obliv_cs = obliv_c[0].iter().zip(obliv_c[1].iter())
        .zip(obliv_c[2].iter())
        .map(|((c0, c1), c2)| c0.clone()+c1.clone()+c2.clone())
        .collect::<Vec<_>>();
    //let obliv_bs = obliv_b[0].iter().zip(obliv_b[1].iter())
        //.zip(obliv_b[2].iter())
        //.map(|((b0, b1), b2)| b0.clone())
        //.collect::<Vec<_>>();
    //let obliv_cs = obliv_c[0].iter().zip(obliv_c[1].iter())
        //.zip(obliv_c[2].iter())
        //.map(|((c0, c1), c2)| c0.clone())
        //.collect::<Vec<_>>();

    let mut heap = vec![(OblivUInt::new_pub_from_max(num_inputs, num_inputs, 
                                                 state.clone()),
                        (OblivUInt::get_free_zero(state.clone()))); k];
    for (new_id, (b, c)) in obliv_bs.into_iter().zip(obliv_cs.into_iter())
                                    .enumerate() 
    {
        let mut new_tdt = tdt_score(b, c, shifts);
        let mut new_id = OblivUInt::new_pub_from_max(new_id as u64, num_inputs, 
                                                     state.clone());
        for &mut(ref mut id, ref mut tdt) in heap.iter_mut() {
            let gt = new_tdt.obliv_gt(tdt);
            let swap_tdt = OblivUInt::swap(tdt.clone(), new_tdt, gt.clone());
            *tdt = swap_tdt.0;
            new_tdt = swap_tdt.1;
            let swap_id = OblivUInt::swap(id.clone(), new_id, gt);
            *id = swap_id.0;
            new_id = swap_id.1;
            //*id = new_id.clone(); 
            //*tdt = new_tdt.clone(); 
            //break;
        }

    }

    let threshold = OblivUInt::new_pub(threshold, state.clone());
    for &mut (ref mut id, ref mut tdt) in heap.iter_mut() {
        let le = threshold.obliv_le(tdt); 
        *id = id.clone()*le.clone();
        *tdt = tdt.clone()*le;
    }
    for &mut (ref mut id, ref mut obv) in heap.iter_mut() {
       id.obliv_bits = id.obliv_bits.iter().map(|x| x.clone() ^ false).collect();
       obv.obliv_bits = obv.obliv_bits.iter().map(|x| x.clone() ^ false).collect();
    }
    for i in heap {
        let _ = i.0.reveal(0);
        let _ = i.1.reveal(0);
        //println!("ID: {:3}, TDT: {:.4}", i.0.get_debug_value().unwrap(), 
                 //(i.1.get_debug_value().unwrap() as f64)/((2 as u64)
                                                          //.pow(shifts as u32) as f64));
    }

    //state.borrow_mut().report();
    translate(&state.borrow_mut().circuit, &dir);
}


fn tdt_score(b: OblivUInt, c: OblivUInt, shifts: usize) -> OblivUInt {
    let mut ret = (b.clone().into_int()-c.clone().into_int()).pow_uint(2)
               .into_uint();
    let new_size = ret.get_size()+shifts;
    ret.set_size(new_size);
    ret=ret<<shifts;
    ret=ret/(b.clone()+c.clone());
    ret
    //let mut b = b;
    //b.set_size(new_size);
    //b
}

fn gen_input(num_inputs: usize, max: u64, dir: &Path) {
    let mut inputs = Vec::new();
    let mut rng = rand::thread_rng();
    let between = Range::new(0, max+1);
    for _ in 0..num_inputs {
        inputs.push(get_random_six(&between, &mut rng));     
    }
    let mut f0 = File::create(dir.join(Path::new("rand_input_0.txt"))).unwrap(); 
    let mut f1 = File::create(dir.join(Path::new("rand_input_1.txt"))).unwrap(); 
    let mut f2 = File::create(dir.join(Path::new("rand_input_2.txt"))).unwrap(); 
    write!(&mut f0, "SNP b c\n").ok();
    write!(&mut f1, "SNP b c\n").ok();
    write!(&mut f2, "SNP b c\n").ok();
    for (i, ((b0, b1, b2), (c0, c1, c2))) in inputs.into_iter().enumerate() {
        write!(&mut f0, "{} {} {}\n", i, b0, c0).ok();
        write!(&mut f1, "{} {} {}\n", i, b1, c1).ok();
        write!(&mut f2, "{} {} {}\n", i, b2, c2).ok();
    }
}

fn get_random_six(between: &Range<uint_t>, rng_thread: &mut ThreadRng) -> 
    ((uint_t, uint_t, uint_t), (uint_t, uint_t, uint_t))
{
    let mut triples = (0..6).map(|_| between.ind_sample(rng_thread))
                            .collect::<Vec<_>>();
    triples.sort();
    triples = (0..1).chain(triples.into_iter()).collect();
    triples = triples.windows(2).map(|x| x[1]-x[0]).collect::<Vec<_>>();
    ((triples[0], triples[1], triples[2]), 
     (triples[3], triples[4], triples[5]))
}


