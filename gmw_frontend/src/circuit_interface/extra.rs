#![allow(dead_code)]
#![allow(unused_imports)]
use circuit_interface::boolean_circuit_int::*;
use circuit_interface::boolean_circuit::*;

pub fn bubble_sort<T>(vec: &mut Vec<T>, increasing: bool) 
    where T: OblivIntType+Clone+OblivOrd 
{
    for _ in 0..vec.len() {
        for i in 0..(vec.len()-1) {
            let lt = vec[i].obliv_lt(&vec[i+1]);
            let result = T::swap(vec[i].clone(), 
                                 vec[i+1].clone(), increasing ^ lt);
            vec[i] = result.0;
            vec[i+1] = result.1;
        }
    }
}

// swap if OblivBool is true
// note that G must be oblivious
pub fn bubble_sort_with_swap<T,F,G>(vec: &mut Vec<(T, G)>,
                                    increasing: bool, swap: F)
    where T: OblivIntType+Clone+OblivOrd, F: Fn(G, G, OblivBool)->(G, G),
          G: Clone
{
    for _ in 0..vec.len() {
        for i in 0..(vec.len()-1) {
            let lt = vec[i].0.obliv_lt(&vec[i+1].0);
            let result = T::swap(vec[i].0.clone(), 
                                 vec[i+1].0.clone(), increasing ^ lt.clone());
            vec[i].0 = result.0;
            vec[i+1].0 = result.1;
            let result = swap(vec[i].1.clone(), 
                                 vec[i+1].1.clone(), increasing ^ lt.clone());
            vec[i].1 = result.0;
            vec[i+1].1 = result.1;
        }
    }
}


#[cfg(test)]
mod test {
    use super::*; 
    use circuit_interface::boolean_circuit::*;
    use circuit_interface::boolean_circuit_int::*;
    #[test]
    fn test_bubble_sort() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let mut some_ints = vec![
                OblivUInt::new_from_size(Some(1), 10, 1, state.clone()),
                OblivUInt::new_from_size(Some(3), 10, 1, state.clone()),
                OblivUInt::new_from_size(Some(2), 10, 1, state.clone())];
        bubble_sort(&mut some_ints, true);
        assert_eq!(some_ints.iter().map(|x| x.get_debug_value())
                            .collect::<Vec<_>>(),
                   vec![Some(1), Some(2), Some(3)]);
        bubble_sort(&mut some_ints, false);
        assert_eq!(some_ints.iter().map(|x| x.get_debug_value())
                            .collect::<Vec<_>>(),
                   vec![Some(3), Some(2), Some(1)]);
    }
    #[test]
    fn test_bubble_sort_with_swap() {
        let state = new_state(2, 1, Vec::new(), Vec::new());
        let mut some_ints = vec![
                (OblivUInt::new_from_size(Some(1), 10, 1, state.clone()),
                 (OblivUInt::new_from_size(Some(11), 5, 1, state.clone()),
                  OblivUInt::new_from_size(Some(21), 5, 1, state.clone()))),
                (OblivUInt::new_from_size(Some(3), 10, 1, state.clone()),
                 (OblivUInt::new_from_size(Some(13), 5, 1, state.clone()),
                  OblivUInt::new_from_size(Some(23), 5, 1, state.clone()))),
                (OblivUInt::new_from_size(Some(2), 10, 1, state.clone()),
                 (OblivUInt::new_from_size(Some(12), 5, 1, state.clone()),
                  OblivUInt::new_from_size(Some(22), 5, 1, state.clone())))];
        bubble_sort_with_swap(&mut some_ints, true, 
            |(a1, a2), (b1, b2), s: OblivBool| {
                let (c1, d1) = OblivUInt::swap(a1, b1, s.clone()); 
                let (c2, d2) = OblivUInt::swap(a2, b2, s); 
                ((c1, c2), (d1, d2))
            });
        let serialized = some_ints.iter().map(|x| {
            (x.0.get_debug_value(), (x.1).0.get_debug_value(), 
             (x.1).1.get_debug_value())})
                                  .collect::<Vec<_>>();
        assert_eq!(serialized, 
                   vec![(Some(1), Some(11), Some(21)),
                        (Some(2), Some(12), Some(22)),
                        (Some(3), Some(13), Some(23))]);

        bubble_sort_with_swap(&mut some_ints, false, 
            |(a1, a2), (b1, b2), s: OblivBool| {
                let (c1, d1) = OblivUInt::swap(a1, b1, s.clone()); 
                let (c2, d2) = OblivUInt::swap(a2, b2, s); 
                ((c1, c2), (d1, d2))
            });
        let serialized = some_ints.iter().map(|x| {
            (x.0.get_debug_value(), (x.1).0.get_debug_value(), 
             (x.1).1.get_debug_value())})
                                  .collect::<Vec<_>>();
        assert_eq!(serialized, 
                   vec![(Some(3), Some(13), Some(23)),
                        (Some(2), Some(12), Some(22)),
                        (Some(1), Some(11), Some(21))]);
        
    }
}
