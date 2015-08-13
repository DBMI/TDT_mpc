#![allow(unused_imports)]
#![allow(dead_code)]
use circuit_interface::boolean_circuit::*;
use std::cmp::max;
use std::ops::{Add, Sub, Neg, Mul, Div, Shl, Shr, Rem};
use std::ops::{BitXor, BitAnd, BitOr, Not};
use std::fmt;
use std::hash::{Hash};

#[allow(non_camel_case_types)]
pub type int_t = i64;
#[allow(non_camel_case_types)]
pub type uint_t = u64;

#[inline]
fn bits_used_signed(n: int_t) -> usize {
    let size = (64 - (n.abs() as uint_t).leading_zeros() + 1) as usize;
    if size == 0 { 1 } else { size }
}

#[inline]
fn bits_used_unsigned(n: uint_t) -> usize {
    let size = (64 - (n as uint_t).leading_zeros()) as usize;
    if size == 0 { 1 } else { size }
}

fn uint_to_binary(n: uint_t) -> Vec<bool> {
    (0..bits_used_unsigned(n)).map(|x| first_bit_unsigned(n>>x)).collect()
}

fn int_to_binary(n: int_t) -> Vec<bool> {
    (0..bits_used_signed(n)).map(|x| first_bit_signed(n>>x)).collect()
}

#[inline]
fn first_bit_signed(n: int_t) -> bool {
    n.trailing_zeros()==0
}

#[inline]
fn first_bit_unsigned(n: uint_t) -> bool {
    n.trailing_zeros()==0
}

fn uint_to_obliv_bits_pub(n: uint_t, size: usize, state: StateRef) -> 
    Vec<OblivBool>
{
   (0..size).map(|x| OblivBool::new_pub(first_bit_unsigned(n>>x), 
                                         state.clone()))
             .collect()
}

fn uint_to_obliv_bits(n: uint_t, size: usize, party: usize, state: StateRef) -> 
    Vec<OblivBool>
{
   (0..size).map(|x| OblivBool::new(Some(first_bit_unsigned(n>>x)), party,
                                         state.clone()))
             .collect()
}

fn int_to_obliv_bits_pub(n: int_t, size: usize, state: StateRef) -> 
    Vec<OblivBool> 
{
    (0..size).map(|x| OblivBool::new_pub(first_bit_signed(n>>x), 
                                         state.clone()))
             .collect()
}

fn int_to_obliv_bits(n: int_t, size: usize, party: usize, state: StateRef) -> 
    Vec<OblivBool> 
{
    (0..size).map(|x| OblivBool::new(Some(first_bit_signed(n>>x)), party,
                                         state.clone()))
             .collect()
}

fn set_size(obliv_bits: &mut Vec<OblivBool>, new_size: usize) {
        if obliv_bits.len() < new_size {
            let state = obliv_bits[0].get_state();
            let diff = new_size-obliv_bits.len();
            obliv_bits.reserve(diff);
            for _ in 0..diff {
                obliv_bits.push(OblivBool::get_free_false(state.clone()));
            }
            return;
        }
        obliv_bits.truncate(new_size);
}

fn set_size_sign_extended(obliv_bits: &mut Vec<OblivBool>, new_size: usize) {
        if obliv_bits.len() < new_size {
            let diff = new_size-obliv_bits.len();
            let last = obliv_bits.last().unwrap().clone();
            obliv_bits.reserve(diff);
            for _ in 0..diff {
                obliv_bits.push(last.clone());
            }
            return;
        }
        obliv_bits.truncate(new_size);
}

fn curtail_obliv_bits(obliv_bits: &mut Vec<OblivBool>, new_size: usize) {
    if obliv_bits.len() <= new_size { return; }
    let skip = obliv_bits.len() - new_size;
    let new_obliv_bits = obliv_bits.iter().skip(skip).cloned().collect(); 
    *obliv_bits = new_obliv_bits;
}

fn add_to_front(obliv_bits: &mut Vec<OblivBool>, to_add: Vec<OblivBool>) {
        let skip = obliv_bits.len() - to_add.len();
        let new_obliv_bits = {
            let iter = obliv_bits.iter().skip(skip).cloned().zip(to_add.iter());
            let mut cin = OblivBool::get_free_false(obliv_bits[0].get_state());
            let mut sum = Vec::new();
            for (left, right) in iter {
                let (s, cout) = OblivBool::one_bit_add(left.clone(), 
                                                  right.clone(), cin);
                sum.push(s);
                cin = cout;
            } 
            sum
        };
        for i in 0..to_add.len() {
            obliv_bits[i+skip] = new_obliv_bits[i].clone();
        }
}

fn sub_from_front(obliv_bits: &mut Vec<OblivBool>, to_sub: Vec<OblivBool>) {
        let skip = obliv_bits.len() - to_sub.len();
        let new_obliv_bits = {
            let iter = obliv_bits.iter().skip(skip).cloned().zip(to_sub.iter());
            let mut cin = OblivBool::get_free_true(obliv_bits[0].get_state());
            let mut sum = Vec::new();
            for (left, right) in iter {
                let (s, cout) = OblivBool::one_bit_sub(left.clone(), 
                                                  right.clone(), cin);
                sum.push(s);
                cin = cout;
            } 
            sum
        };
        for i in 0..to_sub.len() {
            obliv_bits[i+skip] = new_obliv_bits[i].clone();
        }
}

// if 0 selects a, else selects b
fn one_bit_mux_obliv_bits(a: Vec<OblivBool>, b: Vec<OblivBool>, 
                          s: OblivBool) -> Vec<OblivBool> 
{
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter())
            .map(|(ai, bi)| OblivBool::one_bit_mux(ai.clone(), bi.clone(), 
                                                   s.clone()))
            .collect()
}

// swap on s=true
fn swap_obliv_bits(a: Vec<OblivBool>, b: Vec<OblivBool>, s: OblivBool) ->
    (Vec<OblivBool>, Vec<OblivBool>) 
{
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter())
            .map(|(ai, bi)| OblivBool::one_bit_swap(ai.clone(), bi.clone(), 
                                                   s.clone()))
            .unzip()
}

// count the number of 1's 
fn count_ones_obliv_bits(obliv_bits: &Vec<OblivBool>) -> Vec<OblivBool> 
{
    let shifts = (obliv_bits.len()+1).next_power_of_two(); 
    let mut resized = OblivUInt{ obliv_bits: obliv_bits.clone() };
    resized.set_size(shifts-1);
    count_ones_impl(resized.obliv_bits)
}

fn count_ones_impl(obliv_bits: Vec<OblivBool>) -> Vec<OblivBool> {
    assert!(obliv_bits.len() >= 3);
    if obliv_bits.len() == 3 {
       let (b0, b1) = OblivBool::one_bit_add(
           obliv_bits[0].clone(), obliv_bits[1].clone(), 
           obliv_bits[2].clone());
       return vec![b0, b1];
    }
    let half = obliv_bits.len()/2;
    let sum1 = count_ones_impl( {
        obliv_bits.iter().cloned().take(half).collect()
    });
    let sum2 = count_ones_impl( {
        obliv_bits.iter().cloned().skip(half).take(half).collect()
    });
    let iter = sum1.into_iter().zip(sum2.into_iter());
    let mut cin = obliv_bits.last().unwrap().clone();
    let mut sum = Vec::new();
    for (left, right) in iter {
        let (s, cout) = OblivBool::one_bit_add(left, right, cin);
        sum.push(s);
        cin = cout;
    } 
    sum.push(cin);
    sum
}  

fn leading_bit_from_front(obliv_bits: &Vec<OblivBool>) -> Vec<OblivBool> {
    let mut new_obliv_bits = Vec::with_capacity(obliv_bits.len());
    new_obliv_bits.push(!(obliv_bits.last().unwrap().clone()));
    let mut prev = obliv_bits.last().unwrap().clone();
    for i in 0..(obliv_bits.len()-1) {
        let i = obliv_bits.len()-2-i;
        prev = prev | obliv_bits[i].clone();
        new_obliv_bits.push(!(prev.clone()));
    }
    new_obliv_bits.reverse();
    count_ones_obliv_bits(&new_obliv_bits)
}

//opposite overflow
fn sub_with_overflow_obliv_bits(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> 
    (Vec<OblivBool>, OblivBool)
{
    assert_eq!(lhs.len(), rhs.len());
    let mut cin = OblivBool::get_free_true(lhs[0].get_state());
    let mut diff = Vec::new();
    for (left, right) in lhs.iter().zip(rhs.iter()) {
        let (d, cout) = OblivBool::one_bit_sub(left.clone(), 
                                               right.clone(), cin);
        diff.push(d);
        cin = cout;
    } 
    (diff, cin)
}

fn add_with_overflow_obliv_bits(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> 
    (Vec<OblivBool>, OblivBool)
{
    assert_eq!(lhs.len(), rhs.len());
    let mut cin = OblivBool::get_free_false(lhs[0].get_state());
    let mut sum = Vec::new();
    for (left, right) in lhs.iter().zip(rhs.iter()) {
        let (s, cout) = OblivBool::one_bit_add(left.clone(), 
                                               right.clone(), cin);
        sum.push(s);
        cin = cout;
    } 
    (sum, cin)
}

fn shl_obliv_bits_usize(lhs: Vec<OblivBool>, rhs: usize) -> Vec<OblivBool> {
    if rhs>=lhs.len() { 
        return vec![OblivBool::get_free_false(lhs[0].get_state()); lhs.len()];
    }
    let mut lhs = lhs;
    for i in 0..lhs.len() {
        let i = lhs.len()-i-1;
        if i<rhs {
            lhs[i] = 
                OblivBool::get_free_false(lhs[0].get_state());
        } 
        else {
           lhs.swap(i, i-rhs);
        }
    }
    lhs
}


fn shr_obliv_bits_usize(lhs: Vec<OblivBool>, rhs: usize) -> Vec<OblivBool> {
    if rhs>=lhs.len() { 
        return vec![OblivBool::get_free_false(lhs[0].get_state()); lhs.len()];
    }
    let mut lhs = lhs;
    for i in 0..lhs.len() {
        if i<lhs.len()-rhs {
           lhs.swap(i,i+rhs);
        } 
        else {
            lhs[i] = OblivBool::get_free_false(lhs[0].get_state());
        }
    }
    lhs
}

fn shl_obliv_bits(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> Vec<OblivBool> {
    let mut result = lhs; 
    for i in 0..rhs.len() {
        let left = result.clone(); 
        let right = shl_obliv_bits_usize(result, (1<<i));
        result = one_bit_mux_obliv_bits(left, right, rhs[i].clone());
    }
    result
}

fn shr_obliv_bits(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> Vec<OblivBool> {
    let mut result = lhs; 
    for i in 0..rhs.len() {
        let left = result.clone(); 
        let right = shr_obliv_bits_usize(result, (1<<i));
        result = one_bit_mux_obliv_bits(left, right, rhs[i].clone());
    }
    result
}

fn mul_by_one_bit_obliv_bits(lhs: OblivBool, rhs: Vec<OblivBool>) -> 
    Vec<OblivBool>
{
    rhs.iter().map(|x| lhs.clone() & x.clone()).collect()
}

fn mul_obliv_bits_unsigned(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> 
    Vec<OblivBool> 
{
    let mut result = mul_by_one_bit_obliv_bits(rhs[0].clone(), lhs.clone());  
    set_size(&mut result, lhs.len()+rhs.len());
    result = shl_obliv_bits_usize(result, rhs.len()-1);
    for i in 1..rhs.len() {
        result = shr_obliv_bits_usize(result, 1);
        let mut temp = lhs.clone(); 
        temp = mul_by_one_bit_obliv_bits(rhs[i].clone(), temp);
        set_size(&mut temp, lhs.len()+1);
        add_to_front(&mut result ,temp);
    }
    result
}

fn mul_obliv_bits_uint(obliv_bits: Vec<OblivBool>, rhs: uint_t) -> 
    Vec<OblivBool> 
{
    let bin_int = uint_to_binary(rhs);
    let mut result = 
        if bin_int[0] {
            let mut temp = obliv_bits.clone();    
            set_size(&mut temp, obliv_bits.len()+bin_int.len());
            temp
        }
        else {
            vec![OblivBool::get_free_false(obliv_bits[0].get_state()); 
                 obliv_bits.len()+bin_int.len()]
        };
    result = shl_obliv_bits_usize(result, bin_int.len()-1);
    for b in bin_int.into_iter().skip(1) {
        result = shr_obliv_bits_usize(result, 1);
        if !b { continue; }
        let mut temp = obliv_bits.clone();
        set_size(&mut temp, obliv_bits.len()+1);
        add_to_front(&mut result, temp);
    }
    result
}

fn mul_obliv_bits_signed(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> 
    Vec<OblivBool> 
{
    let mut result = mul_by_one_bit_obliv_bits(rhs[0].clone(), lhs.clone());  
    set_size_sign_extended(&mut result, lhs.len()+rhs.len()+2);
    result = shl_obliv_bits_usize(result, rhs.len()-1);
    for i in 1..rhs.len() {
        let mut temp = lhs.clone(); 
        temp = mul_by_one_bit_obliv_bits(rhs[i].clone(), temp);
        set_size_sign_extended(&mut temp, lhs.len()+2);
        if i==rhs.len()-1 {
            sub_from_front(&mut result, temp);
        }
        else {
            add_to_front(&mut result ,temp);
        }
        let second_last = result.last().unwrap().clone();
        result = shr_obliv_bits_usize(result, 1);
        *result.last_mut().unwrap() = second_last;
    }
    result
}


//fn mul_obliv_bits_int(obliv_bits: Vec<OblivBool>, rhs: int_t) -> 
    //Vec<OblivBool> 
//{
    //let mut result = vec![OblivBool::get_free_false(obliv_bits[0].get_state());
                          //obliv_bits.len()];
    //let bin_int = int_to_binary(rhs);
    //for (b,shifts) in bin_int.iter().zip(0..(bin_int.len())) {
        //if shifts>= obliv_bits.len() { break; }
        //if !b { continue; }
        //let mut temp = obliv_bits.clone();
        //temp = shl_obliv_bits_usize(temp, shifts);
        //curtail_obliv_bits(&mut temp, obliv_bits.len() - shifts);
        //add_to_front(&mut result, temp);
    //}
    //result
//}

// this assumes lhs has been properly size-increaded by rhs.len()
fn div_obliv_bits_unsigned(lhs: Vec<OblivBool>, rhs: Vec<OblivBool>) -> 
    (Vec<OblivBool>, Vec<OblivBool>, Vec<OblivBool>) 
{
    assert!(lhs.len()>rhs.len());
    let shifts = leading_bit_from_front(&rhs);
    let rhs = shl_obliv_bits(rhs, shifts.clone());
    let lhs = shl_obliv_bits(lhs, shifts.clone());
    let mut lead_digits = lhs.clone();
    curtail_obliv_bits(&mut lead_digits, rhs.len()+1);
    let mut quotient = Vec::new();
    let mut final_diff = Vec::new();
    for i in 0..(lhs.len()-rhs.len()) {
        let mut new_rhs = rhs.clone();
        set_size(&mut new_rhs, lead_digits.len());
        let (diff, cout) = sub_with_overflow_obliv_bits(lead_digits.clone(), 
                                                        new_rhs);
        lead_digits = one_bit_mux_obliv_bits(diff, lead_digits, !cout.clone());
        if i == (lhs.len()-rhs.len()-1) {
            final_diff = lead_digits.clone();
        }
        quotient.push(cout);
        lead_digits = shl_obliv_bits_usize(lead_digits, 1);
        let index = ((lhs.len() as isize) - 
                    (rhs.len() as isize) 
                    - (i as isize) - 2).abs() as usize;
        lead_digits[0] = lhs[index].clone();
    }
    quotient.reverse();
    (quotient, final_diff, shifts)
}

fn div_obliv_bits_unsigned_uint(lhs: Vec<OblivBool>, rhs: uint_t) -> 
    (Vec<OblivBool>, Vec<OblivBool>)
{
    let rhs = OblivUInt::new_pub(rhs, lhs[0].get_state()).obliv_bits;
    if rhs.len()>lhs.len() { 
        return (vec![OblivBool::get_free_false(lhs[0].get_state()); lhs.len()]
                ,lhs);
    }
    let mut lhs = lhs; 
    let new_size = lhs.len() + 1;
    set_size(&mut lhs, new_size);
    let mut lead_digits = lhs.clone();
    curtail_obliv_bits(&mut lead_digits, rhs.len()+1);
    let mut quotient = Vec::new();
    let mut final_diff = Vec::new();
    for i in 0..(lhs.len()-rhs.len()) {
        let mut new_rhs = rhs.clone();
        set_size(&mut new_rhs, lead_digits.len());
        let (diff, cout) = sub_with_overflow_obliv_bits(lead_digits.clone(), 
                                                        new_rhs);
        lead_digits = one_bit_mux_obliv_bits(diff, lead_digits, !cout.clone());
        quotient.push(cout);
        final_diff = lead_digits.clone();
        lead_digits = shl_obliv_bits_usize(lead_digits, 1);
        let index = ((lhs.len() as isize) - 
                    (rhs.len() as isize) 
                    - (i as isize) - 2).abs() as usize;
        lead_digits[0] = lhs[index].clone();
    }
    quotient.reverse();
    (quotient, final_diff)
}

// does not work for signed against unsigned
fn eq_obliv_bits(lhs: &Vec<OblivBool>, rhs: &Vec<OblivBool>) -> OblivBool {
    assert_eq!(lhs.len(), rhs.len());
    !lhs.iter().skip(1).zip(rhs.iter().skip(1))
       .fold(lhs[0].clone() ^ rhs[0].clone() , 
             |acc, (x, y)| (acc|(x.clone()^y.clone())))
    
}

fn gt_obliv_bits_signed(lhs: &Vec<OblivBool>, rhs: &Vec<OblivBool>) -> 
    OblivBool 
{
    assert_eq!(lhs.len(), rhs.len());
    let state = lhs[0].get_state();
    let out = 
        lhs.iter().zip(rhs.iter()).take(lhs.len()-1)
           .fold(OblivBool::get_free_false(state),
           |acc, (x, y)| {
                 x.clone()^((x.clone()^acc.clone())&(y.clone()^acc.clone()))   
           });
    let s = lhs.last().unwrap().clone()^rhs.last().unwrap().clone();
    OblivBool::one_bit_mux(out, rhs.last().unwrap().clone(), s)
} 

fn gt_obliv_bits_unsigned(lhs: &Vec<OblivBool>, rhs: &Vec<OblivBool>) -> OblivBool {
    assert_eq!(lhs.len(), rhs.len());
    let state = lhs[0].get_state();
    lhs.iter().zip(rhs.iter())
       .fold(OblivBool::get_free_false(state),
             |acc, (x, y)| x.clone()^
                ((x.clone()^acc.clone())&(y.clone()^acc.clone())))
} 


fn get_debug_value_binary(obliv_bits: &Vec<OblivBool>) -> String {
    obliv_bits.iter()
              .fold(String::new(),
                     |acc, x| { if x.get_debug_value().unwrap() { 
                                        String::from("1")+&acc } 
                                else { String::from("0")+&acc } } )
}

pub trait OblivIntType {
    fn get_free_zero(state: StateRef) -> Self;
    fn get_free_one(state: StateRef) -> Self;
    fn get_state(&self) -> StateRef;
    fn get_size(&self) -> usize; 
    fn set_size(&mut self, new_size: usize);
    fn make_equal_size(x: &mut Self, y: &mut Self);
    fn mux(lhs: Self, rhs: Self, select: OblivBool) -> Self;
    fn swap(lhs: Self, rhs: Self, select: OblivBool) -> (Self, Self);
}

pub struct OblivUIntOutput {
    bits: Vec<OblivBoolOutput>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct OblivUInt {
    pub obliv_bits: Vec<OblivBool>,
}

impl OblivUInt {
    pub fn new_from_size(value: Option<uint_t>, size: usize,
                     party: usize, state: StateRef) -> Self 
    {
        if size==0 {panic!("size must be at least 1.")}
        let obliv_bits = if value.is_some() {
            uint_to_obliv_bits(value.unwrap(), size, party, state)
        }
        else { 
            (0..size).map(|_| OblivBool::new(None, party, state.clone())).collect()
        };
        OblivUInt{ obliv_bits: obliv_bits }
    } 

    pub fn new_from_max(value: Option<uint_t>, max_range: uint_t, 
                      party: usize, state: StateRef) -> Self 
    {
        if max_range==0 {panic!("max_range must be at least 1.")}
        Self::new_from_size(value, bits_used_unsigned(max_range), party, state)
    }

    pub fn new_pub(value: uint_t, state: StateRef) -> Self {
        let size = bits_used_unsigned(value);
        let obliv_bits = uint_to_obliv_bits_pub(value, size, state);
        OblivUInt{ obliv_bits: obliv_bits }
    }

    pub fn new_pub_from_size(value: uint_t, size: usize, state: StateRef) -> 
        Self 
    {
        let min_size = bits_used_unsigned(value);
        let mut obliv_bits = uint_to_obliv_bits_pub(value, min_size, state);
        set_size(&mut obliv_bits, size);
        OblivUInt{ obliv_bits: obliv_bits }
    }

    pub fn new_pub_from_max(value: uint_t, max_range: uint_t, 
                            state: StateRef) -> Self 
    {
        if max_range==0 { panic!("max_range must be at least 1.") }
        let min_size = bits_used_unsigned(value);
        let mut obliv_bits = uint_to_obliv_bits_pub(value, min_size, state);
        set_size(&mut obliv_bits, bits_used_unsigned(max_range));
        OblivUInt{ obliv_bits: obliv_bits }
    }        
    
    //TODO unit test
    pub fn reveal(&self, party: usize) -> OblivUIntOutput {
        OblivUIntOutput { bits:
            self.obliv_bits.iter().map(|x| x.reveal(party)).collect()
        }
    }

    pub fn set_max(&mut self, new_max: uint_t) {
        self.set_size(bits_used_unsigned(new_max));
    }

    pub fn into_int(self) -> OblivInt {
        let mut mut_self = self;
        let new_size = mut_self.get_size()+1;
        mut_self.set_size(new_size);
        OblivInt { obliv_bits: mut_self.obliv_bits }
    }

    
    pub fn get_debug_value(&self) -> Option<uint_t>{
        if !self.obliv_bits[0].get_debug_value().is_some() {
            return None;
        }
        let start = 0;
        let output = self.obliv_bits
                         .iter()
                         .zip(0..self.get_size())
                         .fold(start, |acc, (item, shift)| {
                            acc + ((item.get_debug_value().unwrap() as uint_t)
                                   <<shift) });
        Some(output)
    }

    pub fn get_debug_value_binary(&self) -> String {
        get_debug_value_binary(&self.obliv_bits)
    }
}

impl OblivIntType for OblivUInt {
    fn get_free_zero(state: StateRef) -> Self {
        OblivUInt{ obliv_bits: vec![OblivBool::get_free_false(state)] }
    }

    fn get_free_one(state: StateRef) -> Self {
        OblivUInt{ obliv_bits: vec![OblivBool::get_free_true(state)] }
    }

    fn get_state(&self) -> StateRef {
        self.obliv_bits[0].get_state()
    }

    fn get_size(&self) -> usize {
        self.obliv_bits.len()
    }

    fn set_size(&mut self, new_size: usize) {
        set_size(&mut self.obliv_bits, new_size);
    }

    fn make_equal_size(x: &mut Self, y: &mut Self) {
        let size = max(x.get_size(), y.get_size());
        x.set_size(size);
        y.set_size(size);
    }

    fn mux(lhs: Self, rhs: Self, select: OblivBool) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;
        Self::make_equal_size(&mut lhs, &mut rhs);
        OblivUInt { obliv_bits: 
            one_bit_mux_obliv_bits(lhs.obliv_bits, rhs.obliv_bits, select)
        }
    }

    fn swap(lhs: Self, rhs: Self, select: OblivBool) -> (Self, Self) {
        let mut lhs = lhs;
        let mut rhs = rhs;
        Self::make_equal_size(&mut lhs, &mut rhs);
        let (a, b) = swap_obliv_bits(lhs.obliv_bits, rhs.obliv_bits, select);
        (OblivUInt { obliv_bits: a }, OblivUInt { obliv_bits: b })
    }
}

impl Shl<OblivUInt> for OblivUInt {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self {
        OblivUInt{ obliv_bits: shl_obliv_bits(self.obliv_bits, 
                                              rhs.obliv_bits) }
    }
}

impl Shl<usize> for OblivUInt {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self {
        OblivUInt{ obliv_bits: shl_obliv_bits_usize(self.obliv_bits, rhs) }
    }
}

impl Shl<OblivUInt> for u64{
    type Output = OblivUInt;
    fn shl(self, rhs: OblivUInt) -> OblivUInt {
        let mut new_self = OblivUInt::new_pub(self, rhs.get_state());
        new_self.set_size(rhs.get_size());
        OblivUInt{ obliv_bits: shl_obliv_bits(new_self.obliv_bits, 
                                              rhs.obliv_bits)}
    }
}

impl Shr<OblivUInt> for OblivUInt {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self {
        OblivUInt{ obliv_bits: shr_obliv_bits(self.obliv_bits, 
                                              rhs.obliv_bits) }
    }
}

impl Shr<usize> for OblivUInt {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self {
        OblivUInt{ obliv_bits: shr_obliv_bits_usize(self.obliv_bits, rhs) }
    }
}

impl Shr<OblivUInt> for u64{
    type Output = OblivUInt;
    fn shr(self, rhs: OblivUInt) -> OblivUInt {
        let mut new_self = OblivUInt::new_pub(self, rhs.get_state());
        new_self.set_size(rhs.get_size());
        OblivUInt{ obliv_bits: shr_obliv_bits(new_self.obliv_bits, 
                                              rhs.obliv_bits)}
    }
}

impl Add for OblivUInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = rhs;
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = add_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
        OblivUInt{ obliv_bits: out }
    }
}

impl Add<uint_t> for OblivUInt {
    type Output = Self;
    fn add(self, rhs: uint_t) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = OblivUInt::new_pub(rhs, mut_self.get_state());
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = add_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
                                                    
        OblivUInt{ obliv_bits: out }
    }
}

impl Add<OblivUInt> for uint_t {
    type Output = OblivUInt;
    fn add(self, rhs: OblivUInt) -> OblivUInt {
        let mut mut_self = OblivUInt::new_pub(self, rhs.get_state());
        let mut mut_rhs = rhs;
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = add_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
                                                    
        OblivUInt{ obliv_bits: out }
    }
}

impl Sub for OblivUInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = rhs;
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = sub_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
        
        OblivUInt{ obliv_bits: out }
    }
}

impl Sub<uint_t> for OblivUInt {
    type Output = Self;
    fn sub(self, rhs: uint_t) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = OblivUInt::new_pub(rhs, mut_self.get_state());
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = sub_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
                                                    
        OblivUInt{ obliv_bits: out }
    }
}

impl Sub<OblivUInt> for uint_t {
    type Output = OblivUInt;
    fn sub(self, rhs: OblivUInt) -> OblivUInt {
        let mut mut_self = OblivUInt::new_pub(self, rhs.get_state());
        let mut mut_rhs = rhs;
        OblivUInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = sub_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
                                                    
        OblivUInt{ obliv_bits: out }
    }
}

impl Mul<OblivUInt> for OblivBool {
    type Output = OblivUInt;
    fn mul(self, rhs: OblivUInt) -> OblivUInt{
        OblivUInt{ obliv_bits: mul_by_one_bit_obliv_bits(self, rhs.obliv_bits)}
    }
}

impl Mul<OblivBool> for OblivUInt {
    type Output = Self;
    fn mul(self, rhs: OblivBool) -> Self {
        OblivUInt{ obliv_bits: mul_by_one_bit_obliv_bits(rhs, self.obliv_bits)}
    }
}

impl Mul for OblivUInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        OblivUInt{ obliv_bits: mul_obliv_bits_unsigned(self.obliv_bits, 
                                                       rhs.obliv_bits) }
    }
}

impl Mul<uint_t> for OblivUInt {
    type Output = Self;
    fn mul(self, rhs: uint_t) -> Self {
        OblivUInt{ obliv_bits: mul_obliv_bits_uint(self.obliv_bits, rhs) }
    }
}

impl Mul<OblivUInt> for uint_t {
    type Output = OblivUInt;
    fn mul(self, rhs: OblivUInt) -> OblivUInt {
        OblivUInt{ obliv_bits: mul_obliv_bits_uint(rhs.obliv_bits, self) }
    }
}

impl Div for OblivUInt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let mut new_self = self;
        let new_size = new_self.get_size() + rhs.get_size();
        new_self.set_size(new_size);
        OblivUInt{ obliv_bits: div_obliv_bits_unsigned(new_self.obliv_bits, 
                                                       rhs.obliv_bits).0 }
    }
}

impl Div<uint_t> for OblivUInt {
    type Output = Self;
    fn div(self, rhs: uint_t) -> Self {
        OblivUInt{ obliv_bits: div_obliv_bits_unsigned_uint(self.obliv_bits, 
                                                            rhs).0 }
    }
}

impl Div<OblivUInt> for uint_t{
    type Output = OblivUInt;
    fn div(self, rhs: OblivUInt) -> OblivUInt {
        let mut new_self = OblivUInt::new_pub(self, rhs.get_state());
        let new_size = new_self.get_size() + rhs.get_size();
        new_self.set_size(new_size);
        new_self/rhs
    }
}

impl Rem for OblivUInt {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        let (_, diff_bits, shifts_bits) = 
            div_obliv_bits_unsigned(self.obliv_bits, rhs.obliv_bits);
        OblivUInt { obliv_bits: diff_bits } >> 
            OblivUInt { obliv_bits: shifts_bits }
    }
}

impl Rem<uint_t> for OblivUInt {
    type Output = Self;
    fn rem(self, rhs: uint_t) -> Self {
       let diff_bits = div_obliv_bits_unsigned_uint(self.obliv_bits, rhs).1;
       OblivUInt { obliv_bits: diff_bits }
    }
}

impl Rem<OblivUInt> for uint_t {
    type Output = OblivUInt;
    fn rem(self, rhs: OblivUInt) -> OblivUInt {
        let mut new_self = OblivUInt::new_pub(self, rhs.get_state());
        let new_size = new_self.get_size() + rhs.get_size();
        new_self.set_size(new_size);
        new_self%rhs
    }
}

pub trait OblivEq<Rhs = Self> {
    fn obliv_eq(&self, other: &Rhs) -> OblivBool ;
    fn obliv_ne(&self, other: &Rhs) -> OblivBool { 
        !Self::obliv_eq(&self, other)
    }
}

impl OblivEq for OblivUInt {
    fn obliv_eq(&self, other: &Self) -> OblivBool {
        if self.get_size()>other.get_size() {
            let mut other = other.clone();
            other.set_size(self.get_size());
            return eq_obliv_bits(&self.obliv_bits, &other.obliv_bits);
        }
        else if self.get_size()<other.get_size() {
            let mut new_self = self.clone();
            new_self.set_size(other.get_size());
            return eq_obliv_bits(&new_self.obliv_bits, &other.obliv_bits);
        }
        eq_obliv_bits(&self.obliv_bits, &other.obliv_bits)        
    }
}

impl OblivEq<uint_t> for OblivUInt {
    fn obliv_eq(&self, other: &uint_t) -> OblivBool {
        if *other==0 {
            let first = self.obliv_bits.first().unwrap().clone();
            return !self.obliv_bits.iter().fold(first, |acc, x| acc|x.clone());
        }
        let state = self.get_state();
        self.obliv_eq(&OblivUInt::new_pub(other.clone(), state))
    }
}

impl OblivEq<OblivUInt> for uint_t {
    fn obliv_eq(&self, other: &OblivUInt) -> OblivBool {
        other.obliv_eq(self)
    }
    
}

pub trait OblivOrd<Rhs = Self> {
    fn obliv_gt(&self, other: &Rhs) -> OblivBool;
    fn obliv_ge(&self, other: &Rhs) -> OblivBool;
    fn obliv_lt(&self, other: &Rhs) -> OblivBool;
    fn obliv_le(&self, other: &Rhs) -> OblivBool;
}

impl OblivOrd for OblivUInt {
    fn obliv_gt(&self, other: &Self) -> OblivBool {
        if self.get_size()>other.get_size() {
            let mut other = other.clone();
            other.set_size(self.get_size());
            return gt_obliv_bits_unsigned(&self.obliv_bits, &other.obliv_bits);
        }
        else if self.get_size()<other.get_size() {
            let mut new_self = self.clone();
            new_self.set_size(other.get_size());
            return gt_obliv_bits_unsigned(&new_self.obliv_bits, 
                                          &other.obliv_bits);
        }
        gt_obliv_bits_unsigned(&self.obliv_bits, &other.obliv_bits)
    }
    fn obliv_ge(&self, other: &Self) -> OblivBool {
        !other.obliv_gt(self)
    }
    fn obliv_lt(&self, other: &Self) -> OblivBool {
        other.obliv_gt(self)
    }
    fn obliv_le(&self, other: &Self) -> OblivBool {
        !self.obliv_gt(other)
    }
}

impl OblivOrd<uint_t> for OblivUInt {
    fn obliv_gt(&self, other: &uint_t) -> OblivBool {
        if bits_used_unsigned(*other)>self.get_size() {
            return OblivBool::get_free_false(self.get_state());
        }
        let state = self.get_state();
        self.obliv_gt(&OblivUInt::new_pub(*other, state))
    }
    fn obliv_ge(&self, other: &uint_t) -> OblivBool {
        if *other==0 {
            return OblivBool::get_free_true(self.get_state());
        }
        if bits_used_unsigned(*other)>self.get_size() {
            return OblivBool::get_free_false(self.get_state());
        }
        let state = self.get_state();
        self.obliv_ge(&OblivUInt::new_pub(*other, state))
    }
    fn obliv_lt(&self, other: &uint_t) -> OblivBool {
        if *other==0 {
            return OblivBool::get_free_false(self.get_state());
        }
        let state = self.get_state();
        self.obliv_lt(&OblivUInt::new_pub(*other, state))
    }
    fn obliv_le(&self, other: &uint_t) -> OblivBool {
        let state = self.get_state();
        self.obliv_le(&OblivUInt::new_pub(*other, state))
    }
}

impl OblivOrd<OblivUInt> for uint_t {
    fn obliv_gt(&self, other: &OblivUInt) -> OblivBool {
        other.obliv_lt(self)
    } 
    fn obliv_ge(&self, other: &OblivUInt) -> OblivBool {
        other.obliv_le(self)
    } 
    fn obliv_lt(&self, other: &OblivUInt) -> OblivBool {
        other.obliv_gt(self)
    } 
    fn obliv_le(&self, other: &OblivUInt) -> OblivBool {
        other.obliv_ge(self)
    } 
}

impl fmt::Debug for OblivUInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}]", self.get_debug_value())    
    }
}

pub struct OblivIntOutput {
    bits: Vec<OblivBoolOutput>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct OblivInt {
    obliv_bits: Vec<OblivBool>,
}

impl OblivInt {
    pub fn new_from_size(value: Option<int_t>, size: usize,
                     party: usize, state: StateRef) -> Self 
    {
        if size==0 {panic!("size must be at least 1.")}
        let obliv_bits = if value.is_some() {
            int_to_obliv_bits(value.unwrap(), size, party, state)
        }
        else { 
            (0..size).map(|_| OblivBool::new(None, party, state.clone())).collect()
        };
        OblivInt{ obliv_bits: obliv_bits }
    } 

    pub fn new_from_range(value: Option<int_t>, min_range: int_t,
                      max_range: int_t, party: usize, state: StateRef) -> Self 
    {
        if min_range>=max_range { 
            panic!("max_range must be greater than min_range") 
        }
        Self::new_from_size(value, 
                            max(bits_used_signed(max_range),
                                bits_used_signed(min_range)), 
                            party, state)
    }

    pub fn new_pub(value: int_t, state: StateRef) -> Self {
        let size = bits_used_signed(value);
        let obliv_bits = int_to_obliv_bits_pub(value, size, state);
        OblivInt{ obliv_bits: obliv_bits }
    }

    //TODO unit test
    pub fn reveal(&self, party: usize) -> OblivIntOutput {
        OblivIntOutput { bits:
            self.obliv_bits.iter().map(|x| x.reveal(party)).collect()
        }
    }

    pub fn set_range(&mut self, new_min: int_t, new_max: int_t) {
        if new_min>=new_max { 
            panic!("max_range must be greater than min_range") 
        }
        self.set_size(max(bits_used_signed(new_max), 
                          bits_used_signed(new_min)));
    }

    pub fn into_uint(self) -> OblivUInt {
        let mut mut_self = self;
        let new_size = mut_self.get_size()-1;
        mut_self.set_size(new_size);
        OblivUInt { obliv_bits: mut_self.obliv_bits }
    }

    pub fn pow_uint(self, power: uint_t) -> Self {
        if power==0 {
            return Self::get_free_one(self.get_state());
        }    
        if power==1 {
            return self; 
        }    
        let mut ret = self.clone().pow_uint(power>>1)*
            self.clone().pow_uint(power>>1);
        if first_bit_unsigned(power) { ret = ret*self.clone() };
        ret
    }

    pub fn get_debug_value(&self) -> Option<int_t>{
        if !self.obliv_bits[0].get_debug_value().is_some() {
            return None;
        }
        let start = 
            if self.obliv_bits.last().unwrap().get_debug_value().unwrap() {
               -(1<<(self.get_size()-1))
            }
            else { 0 };
        let output = self.obliv_bits
                         .iter()
                         .take(self.get_size()-1)
                         .zip(0..(self.get_size()-1))
                         .fold(start, |acc, (item, shift)| {
                            acc + ((item.get_debug_value().unwrap() as int_t)
                                   <<shift) });
        Some(output)
    }

    pub fn get_debug_value_binary(&self) -> String {
        get_debug_value_binary(&self.obliv_bits)
    }
}

impl OblivIntType for OblivInt {
    fn get_free_zero(state: StateRef) -> Self {
        OblivInt{ obliv_bits: vec![OblivBool::get_free_false(state); 2] }
    }

    fn get_free_one(state: StateRef) -> Self {
        OblivInt{ obliv_bits: vec![OblivBool::get_free_true(state.clone()),
                                    OblivBool::get_free_false(state)] }
    }

    fn get_state(&self) -> StateRef {
        self.obliv_bits[0].get_state()
    }

    fn get_size(&self) -> usize {
        self.obliv_bits.len()
    }

    fn set_size(&mut self, new_size: usize) {
        set_size_sign_extended(&mut self.obliv_bits, new_size);
    }

    fn make_equal_size(x: &mut Self, y: &mut Self) {
        let size = max(x.get_size(), y.get_size());
        x.set_size(size);
        y.set_size(size);
    }

    fn mux(lhs: Self, rhs: Self, select: OblivBool) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;
        Self::make_equal_size(&mut lhs, &mut rhs);
        OblivInt { obliv_bits: 
            one_bit_mux_obliv_bits(lhs.obliv_bits, rhs.obliv_bits, select)
        }
    }

    fn swap(lhs: Self, rhs: Self, select: OblivBool) -> (Self, Self) {
        let mut lhs = lhs;
        let mut rhs = rhs;
        Self::make_equal_size(&mut lhs, &mut rhs);
        let (a, b) = swap_obliv_bits(lhs.obliv_bits, rhs.obliv_bits, select);
        (OblivInt { obliv_bits: a }, OblivInt { obliv_bits: b })
    }
}

impl Shl<OblivUInt> for OblivInt {
    type Output = Self;
    fn shl(self, rhs: OblivUInt) -> Self {
        OblivInt{ obliv_bits: shl_obliv_bits(self.obliv_bits, 
                                              rhs.obliv_bits) }
    }
}

impl Shl<usize> for OblivInt {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self {
        OblivInt{ obliv_bits: shl_obliv_bits_usize(self.obliv_bits, rhs) }
    }
}

impl Shr<OblivUInt> for OblivInt {
    type Output = Self;
    fn shr(self, rhs: OblivUInt) -> Self {
        OblivInt{ obliv_bits: shr_obliv_bits(self.obliv_bits, 
                                              rhs.obliv_bits) }
    }
}

impl Shr<usize> for OblivInt {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self {
        OblivInt{ obliv_bits: shr_obliv_bits_usize(self.obliv_bits, rhs) }
    }
}

impl Add for OblivInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = rhs;
        OblivInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = add_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
                                                    
        OblivInt{ obliv_bits: out }
    }
}

impl Sub for OblivInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut mut_self = self;
        let mut mut_rhs = rhs;
        OblivInt::make_equal_size(&mut mut_self, &mut mut_rhs);
        let (out, _) = sub_with_overflow_obliv_bits(mut_self.obliv_bits,
                                                    mut_rhs.obliv_bits);
        OblivInt{ obliv_bits: out }
    }
}

impl Mul for OblivInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        OblivInt{ obliv_bits: mul_obliv_bits_signed(self.obliv_bits, 
                                                    rhs.obliv_bits) }
    }
}

impl Mul<OblivInt> for OblivBool {
    type Output = OblivInt;
    fn mul(self, rhs: OblivInt) -> OblivInt{
        OblivInt{ obliv_bits: mul_by_one_bit_obliv_bits(self, rhs.obliv_bits)}
    }
}

impl Mul<OblivBool> for OblivInt {
    type Output = Self;
    fn mul(self, rhs: OblivBool) -> Self {
        OblivInt{ obliv_bits: mul_by_one_bit_obliv_bits(rhs, self.obliv_bits)}
    }
}

impl OblivEq for OblivInt {
    fn obliv_eq(&self, other: &Self) -> OblivBool {
        if self.get_size()>other.get_size() {
            let mut other = other.clone();
            other.set_size(self.get_size());
            return eq_obliv_bits(&self.obliv_bits, &other.obliv_bits);
        }
        else if self.get_size()<other.get_size() {
            let mut new_self = self.clone();
            new_self.set_size(other.get_size());
            return eq_obliv_bits(&new_self.obliv_bits, &other.obliv_bits);
        }
        eq_obliv_bits(&self.obliv_bits, &other.obliv_bits)        
    }
}

impl OblivEq<int_t> for OblivInt {
    fn obliv_eq(&self, other: &int_t) -> OblivBool {
        let state = self.get_state();
        self.obliv_eq(&OblivInt::new_pub(other.clone(), state))
    }
}

impl OblivEq<OblivInt> for int_t {
    fn obliv_eq(&self, other: &OblivInt) -> OblivBool {
        let state = other.get_state();
        OblivInt::new_pub(self.clone(), state).obliv_eq(other)
    }
    
}

impl OblivOrd for OblivInt {
    fn obliv_gt(&self, other: &Self) -> OblivBool {
        if self.get_size()>other.get_size() {
            let mut other = other.clone();
            other.set_size(self.get_size());
            return gt_obliv_bits_signed(&self.obliv_bits, &other.obliv_bits);
        }
        else if self.get_size()<other.get_size() {
            let mut new_self = self.clone();
            new_self.set_size(other.get_size());
            return gt_obliv_bits_signed(&new_self.obliv_bits, 
                                        &other.obliv_bits);
        }
        gt_obliv_bits_signed(&self.obliv_bits, &other.obliv_bits)
    }
    fn obliv_ge(&self, other: &Self) -> OblivBool {
        !other.obliv_gt(self)
    }
    fn obliv_lt(&self, other: &Self) -> OblivBool {
        other.obliv_gt(self)
    }
    fn obliv_le(&self, other: &Self) -> OblivBool {
        !self.obliv_gt(other)
    }
}

impl OblivOrd<int_t> for OblivInt {
    fn obliv_gt(&self, other: &int_t) -> OblivBool {
        let state = self.get_state();
        self.obliv_gt(&OblivInt::new_pub(*other, state))
    }
    fn obliv_ge(&self, other: &int_t) -> OblivBool {
        if *other==0 {
            return !self.obliv_bits.last().unwrap().clone();
        }
        let state = self.get_state();
        self.obliv_ge(&OblivInt::new_pub(*other, state))
    }
    fn obliv_lt(&self, other: &int_t) -> OblivBool {
        if *other==0 {
            return self.obliv_bits.last().unwrap().clone();
        }
        let state = self.get_state();
        self.obliv_lt(&OblivInt::new_pub(*other, state))
    }
    fn obliv_le(&self, other: &int_t) -> OblivBool {
        let state = self.get_state();
        self.obliv_le(&OblivInt::new_pub(*other, state))
    }
}

impl OblivOrd<OblivInt> for int_t {
    fn obliv_gt(&self, other: &OblivInt) -> OblivBool {
        other.obliv_lt(self)
    } 
    fn obliv_ge(&self, other: &OblivInt) -> OblivBool {
        other.obliv_le(self)
    } 
    fn obliv_lt(&self, other: &OblivInt) -> OblivBool {
        other.obliv_gt(self)
    } 
    fn obliv_le(&self, other: &OblivInt) -> OblivBool {
        other.obliv_ge(self)
    } 
}


impl fmt::Debug for OblivInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}]", self.get_debug_value())    
    }
}

#[cfg(test)]
mod test {
    use super::*; 
    use circuit_interface::boolean_circuit::*;
    #[test]
    fn test_count_ones () {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 11, 1, state.clone());
        assert_eq!(OblivUInt{ 
            obliv_bits: super::count_ones_obliv_bits(&a.obliv_bits) }
                   .get_debug_value().unwrap(), 6);
        let a = OblivUInt::new_from_size(Some(258), 9, 1, state.clone());
        assert_eq!(OblivUInt{ 
            obliv_bits: super::count_ones_obliv_bits(&a.obliv_bits) }
                   .get_debug_value().unwrap(), 2);
        let a = OblivUInt::new_from_size(Some(0), 9, 1, state.clone());
        assert_eq!(OblivUInt{ 
            obliv_bits: super::count_ones_obliv_bits(&a.obliv_bits) }
                   .get_debug_value().unwrap(), 0);
    }
    #[test]
    fn test_leading_bit() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!(OblivUInt{ 
            obliv_bits: super::leading_bit_from_front(&a.obliv_bits)}
            .get_debug_value().unwrap(), 5);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!(OblivUInt{ 
            obliv_bits: super::leading_bit_from_front(&a.obliv_bits)}
            .get_debug_value().unwrap(), 0);
    }
    #[test]
    fn test_one_bit_mux() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(100), 10, 2, state.clone());
        let s = OblivBool::new_pub(false, state.clone());
        let out = super::one_bit_mux_obliv_bits(a.obliv_bits.clone(), 
                                                b.obliv_bits.clone(), s); 
        assert_eq!(OblivUInt{ obliv_bits: out }.get_debug_value().unwrap(), 
                   1000);
        let s = OblivBool::new_pub(true, state.clone());
        let out = super::one_bit_mux_obliv_bits(a.obliv_bits.clone(), 
                                                b.obliv_bits.clone(), s); 
        assert_eq!(OblivUInt{ obliv_bits: out }.get_debug_value().unwrap(), 
                   100);
    }

    #[test]
    fn test_swap() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivInt::new_from_size(Some(-1000), 12, 1, state.clone());
        let b = OblivInt::new_from_size(Some(100), 7, 2, state.clone());
        let s = OblivBool::new_pub(true, state.clone());
        let (new_a, new_b)= OblivInt::swap(a.clone(), b.clone(), s);
        assert_eq!((a.get_debug_value().unwrap(), 
                    b.get_debug_value().unwrap()),
                   (new_b.get_debug_value().unwrap(), 
                    new_a.get_debug_value().unwrap()));
        let a = OblivInt::new_from_size(Some(-1000), 12, 1, state.clone());
        let b = OblivInt::new_from_size(Some(100), 7, 2, state.clone());
        let s = OblivBool::new_pub(false, state.clone());
        let (new_a, new_b)= OblivInt::swap(a.clone(), b.clone(), s);
        assert_eq!((a.get_debug_value().unwrap(), 
                    b.get_debug_value().unwrap()),
                   (new_a.get_debug_value().unwrap(), 
                    new_b.get_debug_value().unwrap()));
    }

    #[test]
    fn test_sub_with_overflow() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(100), 10, 2, state.clone());
        let (out, overflow) = 
            super::sub_with_overflow_obliv_bits(a.obliv_bits.clone(), 
                                                b.obliv_bits.clone()); 
        let out = OblivUInt{ obliv_bits: out };
        assert_eq!(out.get_debug_value().unwrap(), 900);
        assert_eq!(overflow.get_debug_value().unwrap(), true);
        let (_, overflow) = 
            super::sub_with_overflow_obliv_bits(b.obliv_bits, 
                                                a.obliv_bits);
        assert_eq!(overflow.get_debug_value().unwrap(), false);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(100), 9, 2, state.clone());
        assert_eq!((a-b).get_debug_value().unwrap(), 900);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!((a-0u64).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(100), 10, 1, state.clone());
        assert_eq!((1000u64-a).get_debug_value().unwrap(), 900);

        let a = OblivInt::new_from_size(Some(1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((b-a).get_debug_value().unwrap(), -990);
        let a = OblivInt::new_from_size(Some(1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a-b).get_debug_value().unwrap(), 990);
        let a = OblivInt::new_from_size(Some(0), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a-b).get_debug_value().unwrap(), -10);
        let a = OblivInt::new_from_size(Some(0), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(0), 9, 2, state.clone());
        assert_eq!((a-b).get_debug_value().unwrap(), 0);
        let a = OblivInt::new_from_size(Some(-100), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 9, 2, state.clone());
        assert_eq!((a-b).get_debug_value().unwrap(), -90);

    }
    #[test]
    fn test_add_with_overflow() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(10), 10, 2, state.clone());
        let c = OblivUInt::new_from_size(Some(100), 10, 2, state.clone());
        let (out, overflow) = 
            super::add_with_overflow_obliv_bits(a.obliv_bits.clone(), 
                                                b.obliv_bits.clone());
        let out = OblivUInt{ obliv_bits: out };
        assert_eq!(out.get_debug_value().unwrap(), 1010);
        assert_eq!(overflow.get_debug_value().unwrap(), false);
        let (_, overflow) = 
            super::add_with_overflow_obliv_bits(a.obliv_bits, 
                                                c.obliv_bits);
        assert_eq!(overflow.get_debug_value().unwrap(), true);

        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 1010);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 10);
        let a = OblivUInt::new_from_size(Some(0), 1, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(0), 1, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 0);

        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!((a+0u64).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!((10u64+a).get_debug_value().unwrap(), 1010);

        let a = OblivInt::new_from_size(Some(1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 1010);
        let a = OblivInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 10);
        let a = OblivInt::new_from_size(Some(0), 2, 1, state.clone());
        let b = OblivInt::new_from_size(Some(0), 2, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 0);
        let a = OblivInt::new_from_size(Some(1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), 990);
        let a = OblivInt::new_from_size(Some(-1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), -1010);
        let a = OblivInt::new_from_size(Some(-1000), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), -990);
        let a = OblivInt::new_from_size(Some(0), 11, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 9, 2, state.clone());
        assert_eq!((a+b).get_debug_value().unwrap(), -10);

    }

    #[test]
    fn test_mul_by_one_bit() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivBool::get_free_false(state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivBool::get_free_true(state.clone());
        assert_eq!((b*a.clone()).get_debug_value().unwrap(), 
                   a.get_debug_value().unwrap());
    }

    #[test]
    fn test_shl() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a<<b).get_debug_value().unwrap(), 8000);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(0), 5, 2, state.clone());
        assert_eq!((a<<b).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(0), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a<<b).get_debug_value().unwrap(), 0);
        let b = OblivUInt::new_from_size(Some(3), 15, 2, state.clone());
        assert_eq!((1000<<b).get_debug_value().unwrap(), 8000);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a<<0).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a<<3).get_debug_value().unwrap(), 8000);

        let a = OblivInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a<<b).get_debug_value().unwrap(), 8000);
        let a = OblivInt::new_from_size(Some(-1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a<<b).get_debug_value().unwrap(), -8000);
        let a = OblivInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a<<3).get_debug_value().unwrap(), 8000);
        let a = OblivInt::new_from_size(Some(-1000), 15, 1, state.clone());
        assert_eq!((a<<3).get_debug_value().unwrap(), -8000);
    }

    #[test]
    fn test_shr() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a>>b).get_debug_value().unwrap(), 125);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(0), 5, 2, state.clone());
        assert_eq!((a>>b).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(0), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a>>b).get_debug_value().unwrap(), 0);
        let b = OblivUInt::new_from_size(Some(3), 15, 2, state.clone());
        assert_eq!((1000>>b).get_debug_value().unwrap(), 125);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a>>0).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a>>3).get_debug_value().unwrap(), 125);

        let a = OblivInt::new_from_size(Some(1000), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 2, state.clone());
        assert_eq!((a>>b).get_debug_value().unwrap(), 125);
        let a = OblivInt::new_from_size(Some(1000), 15, 1, state.clone());
        assert_eq!((a>>3).get_debug_value().unwrap(), 125);
    }

    #[test]
    fn test_mul() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(198), 15, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(37), 10, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 198*37);
        let a = OblivUInt::new_from_size(Some(0), 1, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(37), 15, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(1), 1, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(37), 15, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 37);

        let a = OblivInt::new_from_size(Some(198), 15, 1, state.clone());
        let b = OblivInt::new_from_size(Some(370), 15, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 198*370);
        let a = OblivInt::new_from_size(Some(-198), 18, 1, state.clone());
        let b = OblivInt::new_from_size(Some(37), 10, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), -198*37);
        let a = OblivInt::new_from_size(Some(-198), 20, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-37), 10, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 198*37);

        let a = OblivInt::new_from_size(Some(0), 2, 1, state.clone());
        let b = OblivInt::new_from_size(Some(37), 15, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 0);
        let a = OblivInt::new_from_size(Some(1), 2, 1, state.clone());
        let b = OblivInt::new_from_size(Some(37), 15, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), 37);
        let a = OblivInt::new_from_size(Some(1), 2, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-1), 2, 2, state.clone());
        assert_eq!((a*b).get_debug_value().unwrap(), -1);
    }

    #[test]
    fn test_mul_const() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(198), 15, 1, state.clone());
        assert_eq!((a*(37u64)).get_debug_value().unwrap(), 198*37);
        let a = OblivUInt::new_from_size(Some(198), 15, 1, state.clone());
        assert_eq!((0u64 *a).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(198), 15, 1, state.clone());
        assert_eq!((a*1u64).get_debug_value().unwrap(), 198);
    }

    #[test]
    fn test_div() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 1, state.clone());
        assert_eq!((a/b).get_debug_value().unwrap(), 333);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((a/b).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(1), 1, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((a/b).get_debug_value().unwrap(), 1);
        let a = OblivUInt::new_from_size(Some(0), 1, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((a/b).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!((a/3u64).get_debug_value().unwrap(), 333);
        let a = OblivUInt::new_from_size(Some(1000), 10, 1, state.clone());
        assert_eq!((a/1u64).get_debug_value().unwrap(), 1000);
        let a = OblivUInt::new_from_size(Some(1), 1, 1, state.clone());
        assert_eq!((a/1u64).get_debug_value().unwrap(), 1);
        let a = OblivUInt::new_from_size(Some(0), 1, 1, state.clone());
        assert_eq!((a/1u64).get_debug_value().unwrap(), 0);
        let b = OblivUInt::new_from_size(Some(3), 5, 1, state.clone());
        assert_eq!((1000/b).get_debug_value().unwrap(), 333);
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((1000/b).get_debug_value().unwrap(), 1000);
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((1/b).get_debug_value().unwrap(), 1);
        let b = OblivUInt::new_from_size(Some(1), 5, 1, state.clone());
        assert_eq!((0/b).get_debug_value().unwrap(), 0);
    }

    #[test]
    fn test_rem() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(17), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(3), 5, 1, state.clone());
        assert_eq!((a%b).get_debug_value().unwrap(), 2);
        let a = OblivUInt::new_from_size(Some(8), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(19), 5, 1, state.clone());
        assert_eq!((a%b).get_debug_value().unwrap(), 8);
        let a = OblivUInt::new_from_size(Some(116), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(2), 5, 1, state.clone());
        assert_eq!((a%b).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(17), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(1), 5, 0, state.clone());
        assert_eq!((a%b).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(10), 5, 1, state.clone());
        assert_eq!((a%b).get_debug_value().unwrap(), 0);

        let a = OblivUInt::new_from_size(Some(17), 10, 1, state.clone());
        assert_eq!((a%3).get_debug_value().unwrap(), 2);
        let a = OblivUInt::new_from_size(Some(8), 10, 1, state.clone());
        assert_eq!((a%19).get_debug_value().unwrap(), 8);
        let a = OblivUInt::new_from_size(Some(116), 10, 1, state.clone());
        assert_eq!((a%2).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(17), 10, 1, state.clone());
        assert_eq!((a%1).get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        assert_eq!((a%10).get_debug_value().unwrap(), 0);

        let b = OblivUInt::new_from_size(Some(3), 5, 1, state.clone());
        assert_eq!((17%b).get_debug_value().unwrap(), 2);
        let b = OblivUInt::new_from_size(Some(19), 5, 1, state.clone());
        assert_eq!((8%b).get_debug_value().unwrap(), 8);
        let b = OblivUInt::new_from_size(Some(2), 5, 1, state.clone());
        assert_eq!((116%b).get_debug_value().unwrap(), 0);
        let b = OblivUInt::new_from_size(Some(1), 5, 0, state.clone());
        assert_eq!((17%b).get_debug_value().unwrap(), 0);
        let b = OblivUInt::new_from_size(Some(10), 5, 1, state.clone());
        assert_eq!((0%b).get_debug_value().unwrap(), 0);
    }

    #[test] 
    fn test_eq() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(112), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((a.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(111), 14, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((a.obliv_ne(&b)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(0), 13, 1, state.clone());
        assert_eq!((a.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(112), 10, 1, state.clone());
        assert_eq!((a.obliv_eq(&112)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(111), 14, 1, state.clone());
        assert_eq!((a.obliv_ne(&112)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        assert_eq!((a.obliv_eq(&0)).get_debug_value().unwrap(), true);
        let b = OblivUInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((112.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let b = OblivUInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((111.obliv_ne(&b)).get_debug_value().unwrap(), true);
        let b = OblivUInt::new_from_size(Some(0), 13, 1, state.clone());
        assert_eq!((0.obliv_eq(&b)).get_debug_value().unwrap(), true);


        let a = OblivInt::new_from_size(Some(112), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((a.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(111), 14, 1, state.clone());
        let b = OblivInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((a.obliv_ne(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(0), 13, 1, state.clone());
        assert_eq!((a.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(112), 10, 1, state.clone());
        assert_eq!((a.obliv_eq(&112)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(111), 14, 1, state.clone());
        assert_eq!((a.obliv_ne(&112)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(0), 10, 1, state.clone());
        assert_eq!((a.obliv_eq(&0)).get_debug_value().unwrap(), true);
        let b = OblivInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((112.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let b = OblivInt::new_from_size(Some(112), 13, 1, state.clone());
        assert_eq!((111.obliv_ne(&b)).get_debug_value().unwrap(), true);
        let b = OblivInt::new_from_size(Some(0), 13, 1, state.clone());
        assert_eq!((0.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(-112), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-112), 13, 1, state.clone());
        assert_eq!((a.obliv_eq(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(-111), 14, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-112), 13, 1, state.clone());
        assert_eq!((a.obliv_ne(&b)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(112), 14, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-112), 13, 1, state.clone());
        assert_eq!((a.obliv_ne(&b)).get_debug_value().unwrap(), true);
    }

    #[test]
    fn test_core_eq() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivInt::new_pub(10, state.clone());
        let b = OblivInt::new_pub(10, state.clone());
        let c = OblivInt::new_from_range(Some(10), 0, 10, 1, state.clone());
        let d = OblivInt::new_from_range(Some(10), 0, 10, 1, state.clone());
        assert_eq!(a.clone(), a.clone());
        assert_eq!(a.clone(), b.clone());
        assert!(a.clone()!=c.clone());
        assert!(c.clone()!=d.clone());
    }

    #[test]
    fn test_ord() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(10), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(9), 13, 1, state.clone());
        assert_eq!((a.obliv_gt(&b)).get_debug_value().unwrap(), true);
        assert_eq!((b.obliv_lt(&a)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        let b = OblivUInt::new_from_size(Some(0), 13, 1, state.clone());
        assert_eq!((a.obliv_ge(&b)).get_debug_value().unwrap(), true);
        assert_eq!((b.obliv_le(&a)).get_debug_value().unwrap(), true);

        let a = OblivUInt::new_from_size(Some(10), 10, 1, state.clone());
        assert_eq!((a.obliv_gt(&9)).get_debug_value().unwrap(), true);
        assert_eq!((9.obliv_lt(&a)).get_debug_value().unwrap(), true);
        let a = OblivUInt::new_from_size(Some(0), 10, 1, state.clone());
        assert_eq!((a.obliv_ge(&0)).get_debug_value().unwrap(), true);
        assert_eq!((0.obliv_le(&a)).get_debug_value().unwrap(), true);

        let a = OblivInt::new_from_size(Some(9), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 13, 1, state.clone());
        assert_eq!((a.obliv_gt(&b)).get_debug_value().unwrap(), true);
        assert_eq!((b.obliv_lt(&a)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(-9), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-10), 13, 1, state.clone());
        assert_eq!((a.obliv_gt(&b)).get_debug_value().unwrap(), true);
        assert_eq!((b.obliv_lt(&a)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(-1), 10, 1, state.clone());
        let b = OblivInt::new_from_size(Some(-1), 13, 1, state.clone());
        assert_eq!((a.obliv_ge(&b)).get_debug_value().unwrap(), true);
        assert_eq!((b.obliv_le(&a)).get_debug_value().unwrap(), true);

        let a = OblivInt::new_from_size(Some(10), 10, 1, state.clone());
        assert_eq!((a.obliv_gt(&9)).get_debug_value().unwrap(), true);
        assert_eq!((9.obliv_lt(&a)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(0), 10, 1, state.clone());
        assert_eq!((a.obliv_ge(&0)).get_debug_value().unwrap(), true);
        assert_eq!((0.obliv_le(&a)).get_debug_value().unwrap(), true);
        let a = OblivInt::new_from_size(Some(-1), 10, 1, state.clone());
        assert_eq!((a.obliv_ge(&0)).get_debug_value().unwrap(), false);
        assert_eq!((0.obliv_le(&a)).get_debug_value().unwrap(), false);
    }

    #[test]
    fn test_uint() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivUInt::new_from_size(Some(255), 8, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 255);
        let a = OblivUInt::new_from_size(Some(256), 8, 1, state.clone());
        assert!(a.get_debug_value().unwrap()!=256);
        let a = OblivUInt::new_from_size(Some(0), 1, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_size(Some(1), 1, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 1);
        let a = OblivUInt::new_from_max(Some(38573), 38573, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 38573);
        let a = OblivUInt::new_from_max(Some(0), 1, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 0);
        let a = OblivUInt::new_from_max(Some(2), 1, 1, state.clone());
        assert!(a.get_debug_value().unwrap()!=2);
        let a = OblivUInt::new_from_size(Some(255), 8, 1, state.clone())
                    .into_int();
        assert_eq!(a.get_debug_value().unwrap(), 255);
    }

    #[test]
    fn test_int() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivInt::new_from_size(Some(255), 9, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 255);
        let a = OblivInt::new_from_size(Some(256), 8, 1, state.clone());
        assert!(a.get_debug_value().unwrap()!=256);
        let a = OblivInt::new_from_size(Some(-256), 9, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), -256);
        let a = OblivInt::new_from_size(Some(-257), 9, 1, state.clone());
        assert!(a.get_debug_value().unwrap()!=-257);
        let a = OblivInt::new_from_size(Some(0), 1, 1, state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 0);
        let a = OblivInt::new_from_range(Some(235322), 0, 235322, 1, 
                                         state.clone());
        assert_eq!(a.get_debug_value().unwrap(), 235322);
        let a = OblivInt::new_from_range(Some(-235322), -235322, 0, 1, 
                                       state.clone());
        assert_eq!(a.get_debug_value().unwrap(), -235322);
        let a = OblivInt::new_from_range(Some(-0), -235322, 0, 1, 
                                       state.clone());
        assert_eq!(a.get_debug_value().unwrap(), -0);
        let a = OblivInt::new_from_size(Some(255), 9, 1, state.clone())
                    .into_uint();
        assert_eq!(a.get_debug_value().unwrap(), 255);
        let a = OblivInt::new_from_size(Some(-255), 9, 1, state.clone())
                    .into_uint();
        assert!(a.get_debug_value().unwrap() as i64 !=-255);
    }

    #[test]
    fn test_pow() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let a = OblivInt::new_from_size(Some(-19), 6, 1, state.clone());
        assert_eq!(a.clone().pow_uint(0).get_debug_value().unwrap(), 1);
        assert_eq!(a.clone().pow_uint(1).get_debug_value().unwrap(), -19);
        assert_eq!(a.clone().pow_uint(2).get_debug_value().unwrap(), 19*19);
        assert_eq!(a.clone().pow_uint(5).get_debug_value().unwrap(), 
                   -19*19*19*19*19);
    }
    //#[test]
    fn test_interface_int() {
        //let state = new_state();
        //let a = OblivUInt::new_from_size(Some(1000), 15, 1, state.clone());
        //let b = OblivUInt::new_from_size(Some(100), 10, 2, state.clone());
        //state.borrow().debug_show();
    }
}
