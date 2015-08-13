#![allow(dead_code)]
use circuit_desc::circuit_desc_gen::*;
use circuit_desc::gate_desc::*;

#[derive(Clone, Debug)]
pub struct CircuitState<C: CircuitDesc> {
    pub new_wire_label: usize,
    pub new_input_wire_label: usize,
    pub new_output_wire_label: usize,
    pub circuit: C,
}

impl <C: CircuitDesc> CircuitState<C> 
{
    pub fn new(num_parties: usize, this_party: usize, 
               send_sockets: Vec<String>, recv_sockets: Vec<String>) -> Self 
    {
        let circuit = C::new(num_parties, this_party, send_sockets, 
                             recv_sockets);  
        // 0 and 1 are for public bool
        CircuitState { new_wire_label: 2, new_input_wire_label: 2,
                       new_output_wire_label: 0, circuit: circuit }
                    
    }

    pub fn get_new_wire_label(&mut self) -> WireLabel {
        let w = WireLabel(self.new_wire_label);
        self.new_wire_label += 1;
        w
    }

    pub fn get_new_input_wire_label(&mut self) -> InputWireLabel {
        let w = InputWireLabel(self.new_input_wire_label);
        self.new_input_wire_label += 1;
        w
    }

    pub fn get_new_output_wire_label(&mut self) -> OutputWireLabel {
        let w = OutputWireLabel(self.new_output_wire_label);
        self.new_output_wire_label += 1;
        w
    }
}

