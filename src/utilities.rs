pub fn next_combination(mut selection: Vec<usize>, population_size: usize) -> Option<Vec<usize>> {
    let selection_size = selection.len();
    if population_size < selection_size {
        panic!("Cannot get the next combination for a selection size smaller than the population size.");
    }
    let mut i = selection_size - 1;
    while *selection.get(i).unwrap() == population_size - selection_size + i {
        if i == 0 {
            return None;
        }
        i -= 1;
    }
    selection[i] += 1;
    for j in i + 1..selection_size {
        selection[j] = selection[i] + j - i;
    }
    Some(selection)
}

pub fn next_permutation(mut permutation: Vec<usize>) -> Option<Vec<usize>> {
    let mut first = get_first(&permutation)?;
    let mut to_swap = permutation.len() - 1;
    while permutation[first] >= permutation[to_swap] {
        to_swap -= 1;
    }
    swap(&mut permutation, first, to_swap);
    first += 1;
    to_swap = permutation.len() - 1;
    while first < to_swap {
        swap(&mut permutation, first, to_swap);
        first += 1;
        to_swap -= 1;
    }
    Some(permutation)
}
fn get_first(permutation: &Vec<usize>) -> Option<usize> {
    if permutation.len() == 1 {
        return None;
    }
    for index in (0..permutation.len() - 1).rev() {
        if permutation[index] < permutation[index + 1] {
            return Some(index);
        }
    }
    None
}
fn swap(permutation: &mut Vec<usize>, i: usize, j: usize) {
    let tmp = permutation[i];
    permutation[i] = permutation[j];
    permutation[j] = tmp;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_permutation() {
        let mut permutation = Some(vec![0, 1, 2, 3]);
        let mut expected_permutations = [
            Some(vec![0, 1, 2, 3]),
            Some(vec![0, 1, 3, 2]),
            Some(vec![0, 2, 1, 3]),
            Some(vec![0, 2, 3, 1]),
            Some(vec![0, 3, 1, 2]),
            Some(vec![0, 3, 2, 1]),
            Some(vec![1, 0, 2, 3]),
            Some(vec![1, 0, 3, 2]),
            Some(vec![1, 2, 0, 3]),
            Some(vec![1, 2, 3, 0]),
            Some(vec![1, 3, 0, 2]),
            Some(vec![1, 3, 2, 0]),
            Some(vec![2, 0, 1, 3]),
            Some(vec![2, 0, 3, 1]),
            Some(vec![2, 1, 0, 3]),
            Some(vec![2, 1, 3, 0]),
            Some(vec![2, 3, 0, 1]),
            Some(vec![2, 3, 1, 0]),
            Some(vec![3, 0, 1, 2]),
            Some(vec![3, 0, 2, 1]),
            Some(vec![3, 1, 0, 2]),
            Some(vec![3, 1, 2, 0]),
            Some(vec![3, 2, 0, 1]),
            Some(vec![3, 2, 1, 0]),
            None,
        ];
        assert_eq!(permutation, expected_permutations[0]);
        for index in 1..25 {
            permutation = next_permutation(permutation.unwrap());
            assert_eq!(permutation, expected_permutations[index]);
        }
    }

    #[test]
    fn test_next_combination() {
        let mut combination = Some(vec![0, 1, 2, 3]);
        let population_size = 6;
        let expected_combinations = [
            Some(vec!(0, 1, 2, 3)),
            Some(vec!(0, 1, 2, 4)),
            Some(vec!(0, 1, 2, 5)),
            Some(vec!(0, 1, 3, 4)),
            Some(vec!(0, 1, 3, 5)),
            Some(vec!(0, 1, 4, 5)),
            Some(vec!(0, 2, 3, 4)),
            Some(vec!(0, 2, 3, 5)),
            Some(vec!(0, 2, 4, 5)),
            Some(vec!(0, 3, 4, 5)),
            Some(vec!(1, 2, 3, 4)),
            Some(vec!(1, 2, 3, 5)),
            Some(vec!(1, 2, 4, 5)),
            Some(vec!(1, 3, 4, 5)),
            Some(vec!(2, 3, 4, 5)),
            None
        ];
        assert_eq!(combination, expected_combinations[0]);
        for index in 1..16 {
            combination = next_combination(combination.unwrap(), population_size);
            assert_eq!(combination, expected_combinations[index]);
        }
    }
}