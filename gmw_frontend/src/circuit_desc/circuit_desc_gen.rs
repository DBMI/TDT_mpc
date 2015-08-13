#![allow(dead_code)]
extern crate topological_sort;
use circuit_desc::gate_desc::*;
use circuit_desc::gate_desc::BoolGatesDesc::*;
use self::topological_sort::TopologicalSort;
use std::collections::HashMap;
use std::hash::Hash;

const HOST: Party = Party::Party(0);

#[derive(Debug, Clone)]
struct GateCounter {
    n_xor: usize,
    n_and: usize,
    n_or: usize,
    n_not: usize,
    n_feed: usize,
    n_reveal: usize,
}

impl GateCounter {
    fn new() -> Self {
        GateCounter { n_xor: 0, n_and: 0, n_or: 0, n_not: 0, n_feed:0, 
                      n_reveal: 0 }
    }
    fn report(&self) {
        println!("==========REPORT==========");
        println!("#XOR  = {}", self.n_xor);
        println!("#AND  = {}", self.n_and);
        println!("#FEED = {}", self.n_feed);
        println!("#REV  = {}", self.n_reveal);
        println!("==========================");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Party {
    All,
    Party(usize),
}

impl Party {
    pub fn unwrap(self)->usize {
        if let Party::Party(p) = self {
            p
        } 
        else {
            panic!("Party is All.")
        }
    }
}

#[derive(Debug)]
pub struct CircuitInput<T> where T: Eq {
    pub input: InputWireLabel,
    pub value: Option<T>,
    pub party: Party,
}

#[derive(Debug, Clone)]
pub struct CircuitOutput<T> where T:Eq {
    pub output: OutputWireLabel,
    pub value: Option<T>,
    pub party: Party,
}


pub trait CircuitDesc{
    type G: GateDesc + Eq + Hash + Clone;
    type T: Eq + Clone;
    fn new(num_parties: usize, this_party: usize, send_sockets: Vec<String>,
          recv_sockets: Vec<String>) -> Self; 
    fn get_new_gate_id(&mut self) -> usize;
    fn get_inputs(&self) -> &Vec<CircuitInput<Self::T>>;
    fn get_mut_inputs(&mut self) -> &mut Vec<CircuitInput<Self::T>>;
    fn get_outputs(&self) -> &HashMap<OutputWireLabel, CircuitOutput<Self::T>>; 
    fn get_mut_outputs(&mut self) -> 
        &mut HashMap<OutputWireLabel, CircuitOutput<Self::T>>; 
    fn get_circuit(&self) -> &Vec<Self::G>;
    fn get_mut_circuit(&mut self) -> &mut Vec<Self::G>;
    fn get_num_parties(&self) -> usize;
    fn get_this_party(&self) -> usize;
    fn get_num_inputs(&self) -> usize;
    fn incr_feed_count(&mut self);
    fn incr_rev_count(&mut self);

    fn feed(&mut self, output_wire: WireLabel, input: InputWireLabel, 
            value: Option<Self::T>, party: Party) {
        self.get_mut_inputs().push(CircuitInput{ input: input.clone(), 
                                                 value: value,
                                                 party: party });
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.incr_feed_count();
        }
        else {
            self.get_mut_circuit()
                .push(GateDesc::get_feed(GateID(new_id), output_wire, input));
        }
    }

    fn reveal(&mut self, output: OutputWireLabel, input_wire: WireLabel,
             party: Party) -> CircuitOutput<Self::T> {
        let new_output = CircuitOutput{ output: output.clone(),
                                                   value: None,
                                                   party: party };
        self.get_mut_outputs().insert(output.clone(), new_output.clone());
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.incr_rev_count();
        }
        else {
            self.get_mut_circuit()
                .push(GateDesc::get_reveal(GateID(new_id), output, input_wire));
        }
        new_output
    }

    fn get_topsorted_layers(&self)->Vec<Vec<GateID>> {
        let mut layers = Vec::new();
        let mut ts: TopologicalSort<GateID> = TopologicalSort::new();
        for gate in self.get_circuit() {
            if gate.is_feed() {
                use std::usize::MAX;
                ts.add_dependency(GateID(MAX), gate.get_id());
            }
            else {
                let predec_ids = gate.get_predec(self.get_circuit())
                    .iter()
                    .map(|x| x.get_id())
                    .collect::<Vec<_>>();
                for id in predec_ids {
                    ts.add_dependency(id, gate.get_id());
                }
            }
        }
        ts.pop_all();
        loop {
            let layer = ts.pop_all(); 
            if layer.is_empty() {
                break;
            }
            layers.push(layer.iter().cloned().collect());
        }
        layers 
    }
}

#[derive(Debug)]
pub struct BoolCircuitDesc {
    inputs: Vec<CircuitInput<bool>>,
    outputs: HashMap<OutputWireLabel, CircuitOutput<bool>>,
    circuit: Vec<BoolGatesDesc>,
    new_gate_id: usize,
    num_parties: usize,
    this_party: usize,
    send_sockets: Vec<String>,
    recv_sockets: Vec<String>,
    report: GateCounter,
    follow_map: HashMap<WireLabel, Vec<WireLabel>>,
}

impl BoolCircuitDesc {
    pub fn xor(&mut self, output_wire: WireLabel, input_wire_left: WireLabel,
              input_wire_right: WireLabel) {
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.report.n_xor+=1;
        }
        else {
            let is_some = self.follow_map.get(&input_wire_left).is_some();
            if is_some {
                self.follow_map.get_mut(&input_wire_left)
                    .unwrap().push(output_wire.clone());
            }
            else {
                self.follow_map.insert(input_wire_left.clone(),
                                       vec![output_wire.clone()]);
            }
            let is_some = self.follow_map.get(&input_wire_right).is_some();
            if is_some {
                self.follow_map.get_mut(&input_wire_right)
                    .unwrap().push(output_wire.clone());
            }
            else {
                self.follow_map.insert(input_wire_right.clone(),
                                       vec![output_wire.clone()]);
            }

            self.circuit.push(Xor{ gate_id: GateID(new_id), 
                                   output_wire: output_wire,
                                   input_wire_left: input_wire_left, 
                                   input_wire_right: input_wire_right });
        }
    }
    
    pub fn and(&mut self, output_wire: WireLabel, input_wire_left: WireLabel,
              input_wire_right: WireLabel) {
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.report.n_and+=1;
        }
        else {
            let is_some = self.follow_map.get(&input_wire_left).is_some();
            if is_some {
                self.follow_map.get_mut(&input_wire_left)
                    .unwrap().push(output_wire.clone());
            }
            else {
                self.follow_map.insert(input_wire_left.clone(),
                                       vec![output_wire.clone()]);
            }
            let is_some = self.follow_map.get(&input_wire_right).is_some();
            if is_some {
                self.follow_map.get_mut(&input_wire_right)
                    .unwrap().push(output_wire.clone());
            }
            else {
                self.follow_map.insert(input_wire_right.clone(),
                                       vec![output_wire.clone()]);
            }
            self.circuit.push(And{ gate_id: GateID(new_id), 
                                   output_wire: output_wire,
                                   input_wire_left: input_wire_left, 
                                   input_wire_right: input_wire_right });
        }
    }

    pub fn or(&mut self, output_wire: WireLabel, input_wire_left: WireLabel,
              input_wire_right: WireLabel) {
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.report.n_or+=1;
        }
        else {
            self.circuit.push(Or{ gate_id: GateID(new_id), 
                                  output_wire: output_wire,
                                  input_wire_left: input_wire_left, 
                                  input_wire_right: input_wire_right });
        }
    }

    pub fn not(&mut self, output_wire: WireLabel, input_wire: WireLabel) {
        let new_id = self.get_new_gate_id();
        if cfg!(feature="count-gates") {
            self.report.n_not+=1;
        }
        else {
            self.circuit.push(Not{ gate_id: GateID(new_id), 
                                   output_wire: output_wire,
                                   input_wire: input_wire });
        }
    }

    pub fn get_follow_map(&self) -> &HashMap<WireLabel, Vec<WireLabel>> {
       &self.follow_map   
    }

    pub fn report(&self) {
        self.report.report();
    }

    pub fn debug_show(&self) {
        println!("{}", "=============Input=============");
        for i in &self.inputs {
            println!("{:?}", i);
        }
        println!("{}", "=============Output============");
        for i in &self.outputs {
            println!("{:?}", i);
        }
        println!("{}", "=============Circuit===========");
        for i in &self.circuit {
            println!("{:?}", i);
        }
        println!("{}", "===============================");
    }
}

impl CircuitDesc for BoolCircuitDesc {
    type G = BoolGatesDesc;
    type T = bool;
    fn new(num_parties: usize, this_party: usize, send_sockets: Vec<String>,
          recv_sockets: Vec<String>) -> Self 
    {
        let mut bc = BoolCircuitDesc { inputs: Vec::new(), 
                                       outputs: HashMap::new(), 
                                       circuit: Vec::new(), 
                                       new_gate_id: 0,
                                       num_parties: num_parties,
                                       this_party: this_party,
                                       send_sockets: send_sockets,
                                       recv_sockets: recv_sockets, 
                                       report: GateCounter::new(),
                                       follow_map: HashMap::new(),
                                        };
        bc.feed(WireLabel(0), InputWireLabel(0), Some(false), HOST);
        bc.feed(WireLabel(1), InputWireLabel(1), Some(true), HOST);
        bc
    }

    fn get_new_gate_id(&mut self) -> usize {
        let id = self.new_gate_id;
        self.new_gate_id += 1;
        id
    }

    fn get_inputs(&self) -> &Vec<CircuitInput<bool>> { &self.inputs }
    fn get_mut_inputs(&mut self) -> &mut Vec<CircuitInput<bool>> {
        &mut self.inputs
    }
    fn get_outputs(&self) -> &HashMap<OutputWireLabel, CircuitOutput<bool>> { 
        &self.outputs 
    }
    fn get_mut_outputs(&mut self) -> 
        &mut HashMap<OutputWireLabel, CircuitOutput<bool>> { 
        &mut self.outputs 
    }
    fn get_circuit(&self) -> &Vec<BoolGatesDesc> { &self.circuit }
    fn get_mut_circuit(&mut self) -> &mut Vec<BoolGatesDesc> {
        &mut self.circuit
    }
    fn get_num_parties(&self) -> usize {
       self.num_parties 
    }

    fn get_this_party(&self) -> usize {
        self.this_party
    }

    fn get_num_inputs(&self) -> usize {
        self.inputs.len()
    }
    fn incr_feed_count(&mut self) {
       self.report.n_feed+=1; 
    }
    fn incr_rev_count(&mut self) {
       self.report.n_reveal+=1; 
    }

} 

#[cfg(test)]
mod test {
    use super::*;
    use circuit_desc::gate_desc::*;
    use std::collections::HashMap;

    fn follows(a: GateID, b: GateID, 
               follow_map: &HashMap<GateID, Vec<GateID>>) -> bool{
        member_of(b, follow_map.get(&a).unwrap())
    }

    fn precedes(a: GateID, b: GateID, 
                precede_map: &HashMap<GateID, Vec<GateID>>) -> bool{
        member_of(b, precede_map.get(&a).unwrap())
    }

    fn member_of(i: GateID, l: &Vec<GateID>) -> bool {
       l.iter().any(|x| *x==i) 
    }

    #[test]
    #[cfg(not(feature="count-gates"))]
    fn test_layers_bool() {
        use super::Party::*;
        let mut bc = BoolCircuitDesc::new(2, 1, Vec::new(), Vec::new()); 
        bc.feed(WireLabel(2), InputWireLabel(2), Some(true), Party(1));
        bc.feed(WireLabel(3), InputWireLabel(3), Some(false), Party(2));
        bc.xor(WireLabel(4), WireLabel(0), WireLabel(1));
        bc.and(WireLabel(5), WireLabel(2), WireLabel(3));
        bc.or(WireLabel(6), WireLabel(4), WireLabel(5));
        bc.xor(WireLabel(7), WireLabel(4), WireLabel(0));
        bc.xor(WireLabel(8), WireLabel(6), WireLabel(7));
        bc.not(WireLabel(9), WireLabel(8));
        bc.reveal(OutputWireLabel(0), WireLabel(9), All);
        let layers = bc.get_topsorted_layers();
        assert!(member_of(GateID(0), &layers[0]));
        assert!(member_of(GateID(1), &layers[0]));
        assert!(member_of(GateID(2), &layers[0]));
        assert!(member_of(GateID(3), &layers[0]));
        assert!(member_of(GateID(4), &layers[1]));
        assert!(member_of(GateID(5), &layers[1]));
        assert!(member_of(GateID(6), &layers[2]));
        assert!(member_of(GateID(7), &layers[2]));
        assert!(member_of(GateID(8), &layers[3]));
        assert!(member_of(GateID(9), &layers[4]));
    }
}
