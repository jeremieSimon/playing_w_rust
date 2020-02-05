pub fn add_one(xs: Vec<f64>) -> Vec<f64> {
    return xs.iter().map(|x| {x + 1.}).collect();
}

pub fn add_five(xs: Vec<f64>) -> Vec<f64> {
    return xs.iter().map(|x| {x + 5.}).collect();
}

pub fn square(xs: Vec<f64>) -> Vec<f64> {
    return xs.iter().map(|x| {x.powf(2.)}).collect();
}
