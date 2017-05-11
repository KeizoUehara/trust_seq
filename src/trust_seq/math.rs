use std::f64;
use std::i32;

const LANCZOS:[f64;15] = [ 0.99999999999999709182,
                           57.156235665862923517,
                           -59.597960355475491248,
                           14.136097974741747174,
                           -0.49191381609762019978,
                           0.33994649984811888699e-4,
                           0.46523628927048575665e-4,
                           -0.98374475304879564677e-4,
                           0.15808870322491248884e-3,
                           -0.21026444172410488319e-3,
                           0.21743961811521264320e-3,
                           -0.16431810653676389022e-3,
                           0.84418223983852743293e-4,
                           -0.26190838401581408670e-4,
                           0.36899182659531622704e-5,
];

fn calc_log_gamma(x:f64) -> f64{
    if x.is_nan() || x <= 0.0{
        f64::NAN
    }else{
        let g = 607.0/128.0;
        let mut sum = 0.0;
        for i in (1..LANCZOS.len()).rev(){
            sum += LANCZOS[i]/(x+i as f64);
        }
        let half_log_2_pi = 0.5 *(2.0*f64::consts::PI).ln();
        sum += LANCZOS[0];
        let tmp = x+ g+0.5;
        ((x+0.5) * tmp.ln()) - tmp +
            half_log_2_pi + (sum/x).ln()
    }
}
fn calc_log_beta(a:f64,b:f64) ->f64{
    if a.is_nan() || b.is_nan() || a<=0.0 || b <= 0.0 {
        return f64::NAN
    }else{
        calc_log_gamma(a) + calc_log_gamma(b) - calc_log_gamma(a+b)
    }
}
pub fn calc_binomial_distribution_cummulative(number_of_trials:usize,probability_of_success:f64,x:i32)->f64{
    if x < 0{
        0.0
    }else if x >= number_of_trials as i32{
        1.0
    }else {
        1.0 - calc_regularized_beta(
            probability_of_success,
            x as f64 + 1.0,
            number_of_trials as f64- x as f64,1.0e-14,i32::MAX as usize)
    }
}
fn calc_regularized_beta(x:f64,a:f64,b:f64,epsilon:f64,max_iterations:usize) -> f64{
    if x.is_nan() || a.is_nan() || b.is_nan() || x < 0.0 || x > 1.0 || a <=0.0 || b <= 0.0{
        f64::NAN
    }else if x > a * 1.0 / (a+b+2.0){
        1.0 - calc_regularized_beta(1.0 -x,b,a,epsilon,max_iterations)
    }else{
        let get_b = |n:i32,x:f64| {
            if n %2 == 0 {
                let m = n as f64 / 2.0;
                m*(b-m)*x / ((a+2.0*m -1.0)*(a+(2.0*m)))
            }else {
                let m = (n as f64-1.0) / 2.0;
                -((a+m)*(a+b+m)*x)/((a+2.0*m)*(a+2.0*m +1.0))
            }
        };
        let get_a = |n:i32,x:f64| 1.0;
        let eval = calc_continued_func(x,epsilon,max_iterations,get_a,get_b);
        (a*x.ln() + b*(1.0-x).ln() - a.ln() - calc_log_beta(a,b)).exp() / eval
    }
}
fn calc_continued_func<F1, F2>(x: f64,
                               epsilon: f64,
                               max_iterations: usize,
                               get_a: F1,
                               get_b: F2)
                               -> f64
    where F1: Fn(i32, f64) -> f64,
          F2: Fn(i32, f64) -> f64
{
    let mut p0 = 1.0;
    let mut p1: f64 = get_a(0, x);
    let mut q0 = 0.0;
    let mut q1 = 1.0;
    let mut c = p1 / q1;
    let mut n = 0;
    let mut relative_error = f64::MAX;
    while n < max_iterations && relative_error > epsilon {
        n += 1;
        let a = get_a(n as i32, x);
        let b = get_b(n as i32, x);
        let mut p2 = a * p1 + b * p0;
        let mut q2 = a * q1 + b * q0;
        let mut infinite = false;
        if p2.is_infinite() || q2.is_infinite() {
            let mut scale_factor = 1f64;
            let mut last_scale_factor;
            let max_power = 5;
            let scale = a.max(b);
            if scale <= 0.0 {
                panic!("Continued fraction convergents diverged to +/- infinity for value ");
            }
            infinite = true;
            for i in 0..max_power {
                last_scale_factor = scale_factor;
                scale_factor *= scale;
                if a != 0.0 && a > b {
                    p2 = p1 / last_scale_factor + (b / scale_factor * p0);
                    q2 = q1 / last_scale_factor + (b / scale_factor * q0);
                } else if b != 0.0 {
                    p2 = (a / scale_factor * p1) + p0 / last_scale_factor;
                    q2 = (a / scale_factor * q1) + q0 / last_scale_factor;
                }
                infinite = p2.is_infinite() || q2.is_infinite();
                if !infinite {
                    break;
                }
            }
        }
        if infinite {
            panic!("Continued fraction convergents diverged to +/- infinity for value ");
        }
        let r = p2 / q2;
        if r.is_nan() {
            panic!("Continued fraction diverged to NaN for value {0}");
        }
        relative_error = (r / c - 1.0).abs();
        c = p2/q2;
        p0 = p1;
        p1 = p2;
        q0 = q1;
        q1 = q2;
    }
        if n >= max_iterations {
            panic!("Continued fraction convergents failed to converge (in less than {0} iterations) for value {1}");
        }
    return c;
}
#[cfg(test)]
    mod tests{
    use super::*;
    macro_rules! assert_float_eq {
        ($a:expr, $b:expr) => {
            assert_eq!($a.to_string(), $b.to_string());
        }
    }
    #[test]
    fn test_math() {
        let f1 = |n:i32,x:f64| 1.0;
        let f2 = |n:i32,x:f64| 1.0;
        assert_float_eq!(1.6180339887802426,calc_continued_func(
            0.0,
            1e-10,
            10000,
            f1,
            f2));
        assert_float_eq!(0.5723649429247,calc_log_gamma(0.5));
        assert_float_eq!(0.755337203316395,calc_binomial_distribution_cummulative(20,0.4,9));
    }
}
