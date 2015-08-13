use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InputWireLabel(pub usize);
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OutputWireLabel(pub usize);
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WireLabel(pub usize);
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GateID(pub usize);

impl fmt::Debug for InputWireLabel{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let InputWireLabel(i) = *self;
        write!(f, "I({})", i)
    }
}

impl fmt::Debug for OutputWireLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let OutputWireLabel(i) = *self;
        write!(f, "O({})", i)
    }
}

impl fmt::Debug for WireLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let WireLabel(i) = *self;
        write!(f, "W({})", i)
    }
}

impl fmt::Debug for GateID{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let GateID(i) = *self;
        write!(f, "GID({})", i)
    }
}

pub enum WireLabels {
    NoWire,
    OneWire(WireLabel),
    TwoWire(WireLabel, WireLabel),
}

pub trait GateDesc {
    fn get_id(&self) -> GateID;
    fn get_wire_in(&self) -> WireLabels; 
    fn get_wire_out(&self) -> WireLabels; 
    fn get_predec<'a>(&self, others: &'a Vec<Self>) -> Vec<&'a Self>;
    fn is_feed(&self) -> bool;
    fn get_feed(gate_id: GateID, output_wire: WireLabel, 
                input: InputWireLabel) -> Self;
    fn get_reveal(gate_id: GateID, output: OutputWireLabel, 
                input_wire: WireLabel) -> Self;
    fn get_nop() -> Self;
    fn follows(&self, other: &Self) -> bool {
        use self::WireLabels::*;
        match self.get_wire_in() {
            NoWire => false,
            OneWire(w1) => match other.get_wire_out() {
                OneWire(w2) => w1==w2,
                _ => false,
            },
            TwoWire(w11, w12) => match other.get_wire_out() {
                OneWire(w2) => w11==w2 || w12==w2,
                _ => false,
            }
        }
    }

}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BoolGatesDesc {
    Nop,
    Feed{ gate_id: GateID, output_wire: WireLabel, input: InputWireLabel },
    Reveal{ gate_id: GateID, output: OutputWireLabel, input_wire: WireLabel },
    Xor{ gate_id: GateID, output_wire: WireLabel, input_wire_left: WireLabel, 
        input_wire_right: WireLabel },
    And{ gate_id: GateID, output_wire: WireLabel, input_wire_left: WireLabel, 
        input_wire_right: WireLabel },
    Or{ gate_id: GateID, output_wire: WireLabel, input_wire_left: WireLabel, 
        input_wire_right: WireLabel },
    Not{ gate_id: GateID, output_wire: WireLabel, input_wire: WireLabel },
} 

impl GateDesc for BoolGatesDesc {
    fn get_id(&self) -> GateID {
        use self::BoolGatesDesc::*;
        match *self {
            Feed{ ref gate_id, .. } |
                Reveal{ ref gate_id, .. } |
                Xor{ ref gate_id, .. } |
                And{ ref gate_id, .. } |
                Or{ ref gate_id, .. } |
                Not{ ref gate_id, .. }  => gate_id.clone(),
            Nop => panic!("Nop gate appears!"),
        }
    }
    fn get_wire_in(&self) -> WireLabels {
        use self::WireLabels::*;
        use self::BoolGatesDesc::*;
        match *self {
            Feed{..} => NoWire,
            Reveal{ ref input_wire, .. } | Not{ ref input_wire, .. } => 
                OneWire(input_wire.clone()),
            Xor{ ref input_wire_left, ref input_wire_right, .. } |
                And{ ref input_wire_left, ref input_wire_right, .. } |
                Or{ ref input_wire_left, ref input_wire_right, .. } =>
                TwoWire(input_wire_left.clone(), input_wire_right.clone()),
            Nop => panic!("Nop gate appears!")
        }
    }
    fn get_wire_out(&self) -> WireLabels {
        use self::WireLabels::*;
        use self::BoolGatesDesc::*;
        match *self {
            Feed{ ref output_wire, .. } |
                Xor{ ref output_wire, .. } |
                And{ ref output_wire, .. } |
                Or{ ref output_wire, .. } |
                Not{ ref output_wire, .. } => OneWire(output_wire.clone()), 
            Reveal{..} => NoWire,
            Nop => panic!("Nop gate appears!")
        }
    }

    fn get_predec<'a>(&self, others: &'a Vec<Self>) -> Vec<&'a Self> {
        others.iter().filter(|&x| self.follows(x)).collect()
    }

    fn is_feed(&self) -> bool {
        match *self {
            BoolGatesDesc::Feed{..} => true,
            _ => false,
        }
    }

    fn get_feed(gate_id: GateID, output_wire: WireLabel, 
                input: InputWireLabel) -> Self {
        use self::BoolGatesDesc::Feed;
        Feed{ gate_id: gate_id, output_wire: output_wire, input: input }
    }

    fn get_reveal(gate_id: GateID, output: OutputWireLabel, 
                  input_wire: WireLabel) -> Self {
        use self::BoolGatesDesc::Reveal;
        Reveal{ gate_id: gate_id, output: output, input_wire: input_wire }
    }

    fn get_nop() -> Self {
       BoolGatesDesc::Nop 
    }
}

impl fmt::Debug for BoolGatesDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BoolGatesDesc::*;
        match *self {
            Feed{ ref gate_id, ref output_wire, ref input } =>
                write!(f, "{{{:?}: {:?}=>{:?}}}", gate_id, input, output_wire),
            Reveal{ ref gate_id, ref output, ref input_wire } =>
                write!(f, "{{{:?}: {:?}<={:?}}}", gate_id, output, input_wire),
            Xor{ ref gate_id, ref output_wire, ref input_wire_left,
                 ref input_wire_right} =>
                write!(f, "{{{:?}: {:?} ^ {:?} -> {:?}}}", gate_id, 
                       input_wire_left, input_wire_right, output_wire),
            And{ ref gate_id, ref output_wire, ref input_wire_left,
                 ref input_wire_right} =>
                write!(f, "{{{:?}: {:?} & {:?} -> {:?}}}", gate_id, 
                       input_wire_left, input_wire_right, output_wire),
            Or{ ref gate_id, ref output_wire, ref input_wire_left,
                 ref input_wire_right} =>
                write!(f, "{{{:?}: {:?} | {:?} -> {:?}}}", gate_id, 
                       input_wire_left, input_wire_right, output_wire),
            Not{ ref gate_id, ref output_wire, ref input_wire } =>
                write!(f, "{{{:?}: !{:?} -> {:?}}}", gate_id, 
                       input_wire, output_wire),
            Nop => write!(f, "{{Nop}}"),
        }
    }
}

#[cfg(test)]
mod test {
   use super::*;
   #[test]
   fn test_gate_desc() {
       use super::BoolGatesDesc::*;
       let gates = [Feed{ gate_id: GateID(0), output_wire: WireLabel(0), 
                          input: InputWireLabel(0) },
                    Feed{ gate_id: GateID(1), output_wire: WireLabel(1), 
                          input: InputWireLabel(1) },
                    Xor{ gate_id: GateID(2), output_wire: WireLabel(2), 
                         input_wire_left: WireLabel(0),
                         input_wire_right: WireLabel(1) },
                    And{ gate_id: GateID(4), output_wire: WireLabel(3), 
                         input_wire_left: WireLabel(0),
                         input_wire_right: WireLabel(2) },
                    Or{ gate_id: GateID(5), output_wire: WireLabel(4), 
                        input_wire_left: WireLabel(3),
                        input_wire_right: WireLabel(2) },
                    Not{ gate_id: GateID(6), output_wire: WireLabel(5), 
                         input_wire: WireLabel(4) },
                    Reveal{ gate_id: GateID(7), output: OutputWireLabel(0), 
                            input_wire: WireLabel(5) }];
       assert!(gates[2].follows(&gates[0]));
       assert!(gates[2].follows(&gates[1]));
       assert!(gates[3].follows(&gates[0]));
       assert!(gates[3].follows(&gates[2]));
       assert!(gates[4].follows(&gates[3]));
       assert!(gates[4].follows(&gates[2]));
       assert!(gates[5].follows(&gates[4]));
       assert!(gates[6].follows(&gates[5]));
   }
}
