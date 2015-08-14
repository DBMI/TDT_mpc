#![allow(dead_code)]
extern crate gcrypt;
use self::gcrypt::rand::{Random, STRONG_RANDOM};
use circuit_desc::circuit_desc_gen::*;
use circuit_desc::gate_desc::*;
use std::path::Path;
use std::fs::File;
use std::io::Write;
static HOST: usize = 0;
pub static P: &'static str = 
    "8936097950764538541647693880373941060412422053330581106416547022143872696\
    98630961839249214017328690137845116926322878216706524666038184079923573148\
    6130087";  
pub static G: &'static str = 
    "7660915846360464914746169975675514711978996378800371841846530440188167304\
    05064204804594444789817250309402784834892832428277018505212303916770518805\
    7761352";

pub fn translate(circuit_desc: &BoolCircuitDesc, dir: &Path) {
    //let mut address = File::create("chk_tempfiles/addresses.txt").ok().unwrap();
    //for i in 0..circuit_desc.get_ip_address().len() {
        //let _ = write!(&mut address, "{} {}\n", i, circuit_desc.get_ip_address()[i]);
    //} 
    use circuit_desc::gate_desc::BoolGatesDesc::*;
    let token = gcrypt::init(|mut x| {
        x.disable_secmem();
    });
    let mut seed = [0 as u8; 5];
    seed.randomize(token, STRONG_RANDOM);
    let int_seed: u64 = seed.iter().enumerate()
        .fold(0u64, |accu, (i, x)| accu+(*x as u64)<<i*8);
    let config_path = Path::new("config.txt");
    let mut config = File::create(dir.join(config_path)).ok().unwrap();
    write!(&mut config, "num_parties {}\n", 
           circuit_desc.get_num_parties()).ok();
    write!(&mut config, "pid {}\n", circuit_desc.get_this_party()).ok();
    write!(&mut config, "address-book addresses.txt\n").ok();
    write!(&mut config, "input input.txt\n").ok();
    write!(&mut config, "load-circuit circ.bin\n").ok();
    //let num_input = circuit_desc.get_inputs().iter()
        //.fold(0, |acc, x| {
            //if x.party.clone().unwrap()==circuit_desc.get_this_party() { acc+1 }
            //else { acc }
        //});
    let mut num_input = 0;
    for i in circuit_desc.get_inputs() {
        if i.party.clone().unwrap() == circuit_desc.get_this_party() {
            num_input+=1;
        }
    }
    write!(&mut config, "num_input {}\n", num_input -
           if circuit_desc.get_this_party()==HOST { 2 } else { 0 }).ok();
    write!(&mut config, "seed {}\n", int_seed).ok();
    write!(&mut config, "p {}\n", P).ok();
    write!(&mut config, "g {}\n", G).ok();
    let circ_path = Path::new("circ.txt");
    let mut circ = File::create(dir.join(circ_path)).ok().unwrap();
    write!(&mut circ, "n {}\n", circuit_desc.get_num_parties()).ok();
    let mut counter = 0;
    let mut first_non_input = None;
    let mut n_xor = 0;
    let input_path = Path::new("input.txt");
    let mut input = File::create(dir.join(input_path)).ok().unwrap();
    for i in circuit_desc.get_inputs() {
        if (i.input.0==0) | (i.input.0==1) { continue; }
        if i.party.clone().unwrap()==circuit_desc.get_this_party() {
            write!(&mut input, "{} ", if i.value.unwrap() {1} else {0}).ok(); 
        }
    }
    write!(&mut input, "\n").ok();
    for g in circuit_desc.get_circuit() {
        match *g {
            Reveal{ .. } => {},
            Xor{ ref output_wire, .. } => {
                if first_non_input.is_none() {
                    let WireLabel(w) = *output_wire;
                    first_non_input = Some(w);
                } 
                counter+=1;
                n_xor+=1;
            }
            And{ ref output_wire, .. } => 
            {
                if first_non_input.is_none() {
                    let WireLabel(w) = *output_wire;
                    first_non_input = Some(w);
                } 
                counter+=1;
            }
            _ => {counter+=1},
        }; 
    }
    write!(&mut circ, "d {} {} {}\n", counter, first_non_input.unwrap(), 
           n_xor).ok();
    for p in 0..circuit_desc.get_num_parties() {
        let mut first = None;
        let mut last = None;
        for (w, input) in circuit_desc.get_inputs().iter().enumerate() {
            if (input.input.0 == 1) | (input.input.0 == 0) { continue; };
            let unwrapped_p = if let Party::Party(temp) = input.party {
                temp
            }
            else { 0 };
            if p==unwrapped_p {
                if first.is_none() { first=Some(w) };
            }    
            if p < unwrapped_p {
                if last.is_none() { last=Some(w-1) };
                break;
            }    
            if w==circuit_desc.get_inputs().len()-1 {
                last=Some(w);
            }
        }
        if first.is_none() | last.is_none() {
            write!(&mut circ, "i {} {} {}\n", p, 1, 0).ok(); }
        else {
            write!(&mut circ, "i {} {} {}\n", p, first.unwrap(), 
                   last.unwrap()).ok();
        }
    }
    let mut reveal_gates = Vec::new();
    for i in circuit_desc.get_circuit() {
        match *i {
            Reveal {ref input_wire, ref output, .. } => {
                let w = input_wire.0;
                let p = circuit_desc.get_outputs()
                                           .get(output).unwrap().party.clone().unwrap();
                reveal_gates.push((w, p));
            }
            _ => {}
        }
    }
    for p in 0..circuit_desc.get_num_parties() {
        let mut first = None;
        let mut last = None;
        for (i, &(ref w, ref party)) in reveal_gates.iter().enumerate() {
            if p==*party{
                if first.is_none() { first=Some(*w) };
            }    
            if p < *party {
                if last.is_none() { last=Some(*w-1) };
                break;
            }    
            if i==reveal_gates.len()-1 {
                last=Some(*w);
            }
        }
        if first.is_none() | last.is_none() {
            write!(&mut circ, "o {} {} {}\n", p, 1, 0).ok();
        }
        else {
            write!(&mut circ, "o {} {} {}\n", p, first.unwrap(), 
                   last.unwrap()).ok();
        }
    }
    for i in 0..circuit_desc.get_num_parties() {
        write!(&mut circ, "v {} 1\n", i).ok();
    }
    for g in circuit_desc.get_circuit() {
        match *g {
            Xor {ref input_wire_left, ref input_wire_right, 
                ref output_wire, .. } => { 
                    write!(&mut circ, "g {} {} {} {} ", output_wire.0, 2,
                          input_wire_left.0, input_wire_right.0).ok();
                    let dep = circuit_desc.get_follow_map().get(output_wire);
                    if dep.is_none() {
                        write!(&mut circ, "0\n").ok();
                    }
                    else {
                        let dep = dep.unwrap();
                        write!(&mut circ, "{} ", dep.len()).ok();
                        for i in dep {
                            write!(&mut circ, "{} ", i.0).ok();
                        }
                        write!(&mut circ, "\n").ok();
                    }
            },
            And {ref input_wire_left, ref input_wire_right, 
                ref output_wire, .. } => { 
                    write!(&mut circ, "g {} {} {} {} ", output_wire.0, 1,
                          input_wire_left.0, input_wire_right.0).ok();
                    let dep = circuit_desc.get_follow_map().get(output_wire);
                    if dep.is_none() {
                        write!(&mut circ, "0\n").ok();
                    }
                    else {
                        let dep = dep.unwrap();
                        write!(&mut circ, "{} ", dep.len()).ok();
                        for i in dep {
                            write!(&mut circ, "{} ", i.0).ok();
                        }
                        write!(&mut circ, "\n").ok();
                    }
            },
            Feed { ref output_wire, .. } => { 
                    write!(&mut circ, "g {} {} {} {} ", output_wire.0, 0,
                          -1, -1).ok();
                    let dep = circuit_desc.get_follow_map().get(output_wire);
                    if dep.is_none() {
                        write!(&mut circ, "0\n").ok();
                    }
                    else {
                        let dep = dep.unwrap();
                        write!(&mut circ, "{} ", dep.len()).ok();
                        for i in dep {
                            write!(&mut circ, "{} ", i.0).ok();
                        }
                        write!(&mut circ, "\n").ok();
                    }
            },
            _ => {}
        } 
    }
}

#[cfg(test)] 
mod test {
    use super::*;
    use circuit_desc::circuit_desc_gen::*;
    use circuit_interface::boolean_circuit_int::*;
    use circuit_interface::boolean_circuit::*;
    use std::path::Path;

    #[test]
    fn test_write_address() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivBool::new(Some(false), 0, state.clone());
        let b = OblivBool::new(Some(true), 1, state.clone());
        let c = (a ^ b).reveal(0);
        translate(&state.borrow_mut().circuit, &Path::new("test_chk_files"));
    }
}
