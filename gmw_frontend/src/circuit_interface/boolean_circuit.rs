#![allow(dead_code)]
use circuit_desc::circuit_desc_gen::*;
use circuit_desc::gate_desc::*;
use circuit_interface::circuit::*;
use std::ops::{BitXor, BitAnd, BitOr, Not};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};

pub type StateRef = Rc<RefCell<CircuitState<BoolCircuitDesc>>>;
pub fn new_state(num_parties: usize, this_party: usize, 
                 send_sockets: Vec<String>, 
                 recv_sockets: Vec<String>) -> StateRef 
{ 
    Rc::new(RefCell::new(CircuitState::new(num_parties, this_party, 
                                           send_sockets, recv_sockets))) 
}

impl CircuitState<BoolCircuitDesc> {
    pub fn debug_show(&self) {
        self.circuit.debug_show();
    }
    pub fn report(&self) {
        //if !cfg!(feature="count-gates") {
            //println!("#Layers: {}", self.circuit.get_topsorted_layers().len());
        //}
        self.circuit.report();
    }
}

pub type OblivBoolOutput = CircuitOutput<bool>;

#[derive(Clone)]
pub struct OblivBool {
    wire_label: WireLabel,
    state: StateRef,
    debug_value: Option<bool>,
}

impl PartialEq for OblivBool {
    fn eq(&self, other: &Self) -> bool {
        self.wire_label.0 == other.wire_label.0
    }
} 

impl Eq for OblivBool {}

impl Hash for OblivBool {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.wire_label.0.hash(state)
    }
}

impl OblivBool {
    pub fn new(value: Option<bool>, party: usize, state: StateRef) -> Self {
        use circuit_desc::circuit_desc_gen::Party::*;
        let output_wire = state.borrow_mut().get_new_wire_label();
        let input = state.borrow_mut().get_new_input_wire_label();
        state.borrow_mut().circuit.feed(output_wire.clone(), input, value, 
                                        Party(party));
        OblivBool { wire_label: output_wire, state: state, debug_value: value }
    }

    pub fn new_pub(value: bool, state: StateRef) -> Self {
        match value {
            true => OblivBool { wire_label: WireLabel(1), 
                                state: state, 
                                debug_value: Some(value) },
            false => OblivBool { wire_label: WireLabel(0), 
                                state: state, 
                                debug_value: Some(value) }
        }
    }

    pub fn get_free_false(state: StateRef) -> Self {
        Self::new_pub(false, state)
    }

    pub fn get_free_true(state: StateRef) -> Self {
        Self::new_pub(true, state)
    }

    pub fn reveal(&self, party: usize) -> OblivBoolOutput {
        use circuit_desc::circuit_desc_gen::Party::*;
        let output = self.state.borrow_mut()
                         .get_new_output_wire_label();
        self.state.borrow_mut()
            .circuit.reveal(output, self.wire_label.clone(),
                            match party {
                                //0 => All,
                                _ => Party(party)
                            })
    }

    pub fn one_bit_mux(x: OblivBool, y: OblivBool, s: OblivBool) -> OblivBool {
        let tmp = (s ^ true) & (x ^ y.clone());
        y ^ tmp
    }

    pub fn one_bit_swap(x: OblivBool, y: OblivBool, s: OblivBool) -> 
        (OblivBool, OblivBool) 
    {
        let f = s & ( x.clone() ^ y.clone() );
        (f.clone()^x, f^y)
    }
    pub fn one_bit_add(x: OblivBool, y: OblivBool, cin: OblivBool) -> 
        (OblivBool, OblivBool) { // (s, cout)
        let s = y ^ cin.clone();
        let cout = s.clone() & (x.clone() ^ cin.clone());
        (s^x, cout^cin)
    }

    pub fn one_bit_sub(x: OblivBool, y: OblivBool, cin: OblivBool) -> 
        (OblivBool, OblivBool) { // (s, cout)
        let d = y ^ cin.clone();
        let cout = d.clone() & (x.clone() ^ cin.clone());
        (d^x.clone()^true, cout^x)
    }


    pub fn get_state(&self) -> StateRef {
        self.state.clone()
    }

    pub fn get_debug_value(&self) -> Option<bool> {
        self.debug_value
    }
}

impl BitXor for OblivBool {
    type Output = Self;
    fn bitxor(self, rhs: OblivBool) -> OblivBool {
        let output_wire = self.state.borrow_mut().get_new_wire_label();
        self.state.borrow_mut().circuit.xor(output_wire.clone(),
                                                    self.wire_label,
                                                    rhs.wire_label);
        OblivBool { wire_label: output_wire, 
                    state: self.state,
                    debug_value: if self.debug_value.is_some() && 
                                    rhs.debug_value.is_some() {
                        Some(self.debug_value.unwrap() ^ 
                             rhs.debug_value.unwrap()) }
                    else { None }
                  }
    }
}

impl BitAnd for OblivBool {
    type Output = Self;
    fn bitand(self, rhs: OblivBool) -> OblivBool {
        let output_wire = self.state.borrow_mut().get_new_wire_label();
        self.state.borrow_mut().circuit.and(output_wire.clone(),
                                                    self.wire_label,
                                                    rhs.wire_label);
        OblivBool { wire_label: output_wire, 
                    state: self.state,
                    debug_value: if self.debug_value.is_some() && 
                                    rhs.debug_value.is_some() {
                        Some(self.debug_value.unwrap() & 
                             rhs.debug_value.unwrap()) }
                    else { None }
                  }
    }
}

impl BitOr for OblivBool {
    type Output = Self;
    fn bitor(self, rhs: OblivBool) -> OblivBool {
        self.clone() ^ rhs.clone() ^ (self & rhs)
    }
}

impl Not for OblivBool {
    type Output = Self;
    fn not(self) -> OblivBool {
        let state = self.get_state();
        self ^ OblivBool::get_free_true(state)
    }
}

impl BitXor<OblivBool> for bool {
    type Output = OblivBool;
    fn bitxor(self, rhs: OblivBool) -> OblivBool {
        OblivBool::new_pub(self, rhs.get_state()) ^ rhs
    }
}

impl BitAnd<OblivBool> for bool {
    type Output = OblivBool;
    fn bitand(self, rhs: OblivBool) -> OblivBool {
        OblivBool::new_pub(self, rhs.get_state()) & rhs
    }
}

impl BitOr<OblivBool> for bool {
    type Output = OblivBool;
    fn bitor(self, rhs: OblivBool) -> OblivBool {
        OblivBool::new_pub(self, rhs.get_state()) | rhs
    }
}

impl BitXor<bool> for OblivBool {
    type Output = OblivBool;
    fn bitxor(self, rhs: bool) -> OblivBool {
        OblivBool::new_pub(rhs, self.get_state()) ^ self
    }
}

impl BitAnd<bool> for OblivBool {
    type Output = OblivBool;
    fn bitand(self, rhs: bool) -> OblivBool {
        OblivBool::new_pub(rhs, self.get_state()) & self
    }
}

impl BitOr<bool> for OblivBool {
    type Output = OblivBool;
    fn bitor(self, rhs: bool) -> OblivBool {
        OblivBool::new_pub(rhs, self.get_state()) | self
    }
}

impl fmt::Debug for OblivBool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}]", self.get_debug_value())    
    }
}

#[cfg(test)]
mod test {
    use super::*; 
    #[test]
    fn test_interface_bool() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let b = |value, party| OblivBool::new(value, party, state.clone());
        assert!(OblivBool::new(Some(false), 0, state.clone())!=
                OblivBool::new(Some(false), 0, state.clone()));
        assert!(OblivBool::new(Some(false), 0, state.clone())!=
                OblivBool::get_free_false(state.clone()));
        let layer0 = (true, false, b(Some(true), 1), b(Some(true), 2));
        assert_eq!(layer0.2, layer0.2);
        assert_eq!(OblivBool::get_free_false(state.clone()), 
                   OblivBool::get_free_false(state.clone()));
        let layer1 = (layer0.0^layer0.1.clone(), // false 
                      layer0.2.clone()&layer0.3.clone()); // true
        let layer2 = (layer1.0.clone()|layer1.1.clone(), // true
                      layer1.0.clone()^false); // true
        let layer3 = (layer2.0.clone()^layer2.1.clone(),); //false
        let layer4 = (!layer3.0.clone(),); //true
        let output = layer4.0.get_debug_value(); //true
        assert_eq!(output.unwrap(), true);
    }
}
