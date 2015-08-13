use protocols::*;
use types::*;
use gates::*;
use circuit::{BoolCircuit, BoolCircuitDesc, Output};
use std::collections::HashMap;

// Debug Protocol

pub struct DebugProtocol {
    this_party: u32,
}

impl ProtocolDesc for DebugProtocol {
    type VarType = bool;
    type CDescType = BoolCircuitDesc;
    fn new(party: u32)->Self {
        DebugProtocol { this_party: party }
    }

    fn get_party(&self)->u32{
        self.this_party
    }

    fn exec_circuit(&self, circuit: &BoolCircuitDesc)->Vec<(Output, bool)> {
        let mut obliv_inputs = HashMap::new();
        let mut outputs = Vec::new();
        for &(ref input, ref b) in &circuit.inputs {
           obliv_inputs.insert(input.clone(), 
                               DebugOBool::feed(self, b.clone()));
        } 
        outputs
    }
}

// Debug Bool

#[derive(Clone)]
pub struct DebugOBool {
    is_public: bool,
    know_value: bool,
}

#[allow(unused_variables)]
impl OblivVar for DebugOBool {
    type Input = bool;
    type Output = Self;
    type PD = DebugProtocol;
    fn feed(pd: &DebugProtocol, src: bool)->Self {
        DebugOBool { is_public: false, know_value: src } 
    }
    fn reveal(&self, pd: &DebugProtocol)->bool {
        self.know_value
    }
    fn is_public(&self)->bool {
        self.is_public
    }
}

impl OblivBool for DebugOBool {
    
}

#[allow(unused_variables)]
impl OXor for DebugOBool {
    type PD = DebugProtocol;
    type OType = DebugOBool;
    fn o_xor(&self, pd: &DebugProtocol, other: &DebugOBool)->DebugOBool {
       DebugOBool {is_public: false, 
           know_value: self.know_value ^ other.know_value} 
    }
}

#[allow(unused_variables)]
impl OAnd for DebugOBool {
    type PD = DebugProtocol;
    type OType = DebugOBool;
    fn o_and(&self, pd: &DebugProtocol, other: &DebugOBool)->DebugOBool {
       DebugOBool {is_public: false, 
           know_value: self.know_value & other.know_value} 
    }
}

#[allow(unused_variables)]
impl OOr for DebugOBool {
    type PD = DebugProtocol;
    type OType = DebugOBool;
    fn o_or(&self, pd: &DebugProtocol, other: &DebugOBool)->DebugOBool {
        DebugOBool {is_public: false, 
            know_value: self.know_value | other.know_value} 
    }
}

#[allow(unused_variables)]
impl ONot for DebugOBool {
    type PD = DebugProtocol;
    type OType = DebugOBool;
    fn o_not(&self, pd: &DebugProtocol)->DebugOBool {
        let mut temp = self.clone();
        temp.know_value = !temp.know_value;
        temp
    }
    fn o_not_inplace(&mut self, pd: &DebugProtocol) {
        self.know_value = !self.know_value;
    }    
}


#[cfg(test)]
mod test {
    use super::{DebugProtocol, DebugOBool};
    use circuit::*;
    use types::*;
    use protocols::*;
    use gates::*;

    #[test]
    fn test_new_protocol() {
        let pd = DebugProtocol::new(1);
        assert_eq!(pd.get_party(), 1);
    }

    #[test]
    fn test_feed_bool() {
        let pd = DebugProtocol::new(1);
        let a = DebugOBool::feed(&pd, true);
        assert_eq!(a.reveal(&pd), true);
    }

    #[test]
    fn test_gates_bool(){
        let pd = DebugProtocol::new(1);
        let a = DebugOBool::feed(&pd, true);
        let mut b = DebugOBool::feed(&pd, false);
        assert_eq!(b.o_xor(&pd, &a).reveal(&pd), a.know_value ^ b.know_value);
        assert_eq!(b.o_and(&pd, &a).reveal(&pd), a.know_value & b.know_value);
        assert_eq!(b.o_or(&pd, &a).reveal(&pd), a.know_value | b.know_value);
        assert_eq!(b.o_not(&pd).reveal(&pd), !b.know_value);
        let temp = b.clone();
        b.o_not_inplace(&pd);
        assert_eq!(b.reveal(&pd), !temp.know_value);
    }

    #[test]
    fn test_exec_circuit(){
        let mut circuit = BoolCircuit::new();
        circuit.feed(&Label(2), true);
        circuit.feed(&Label(3), false);
        circuit.feed(&Label(4), true);
        circuit.feed(&Label(5), false);
        circuit.xor(&Label(6), &Label(0), &Label(1));
        circuit.and(&Label(7), &Label(2), &Label(3));
        circuit.or(&Label(8), &Label(4), &Label(5));
        circuit.xor(&Label(9), &Label(6), &Label(7));
        circuit.not(&Label(10), &Label(8));
        circuit.xor(&Label(11), &Label(9), &Label(10));
        circuit.reveal(&Label(11));
        circuit.debug_show_circuit();
    }

}


