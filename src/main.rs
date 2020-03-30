use std::io::Write;
use vob::Vob;

type Num = usize;
// The number base to use. 10 for decimal.
const BASE: Num = 10;
// The number of digits.
const N_DIGITS: usize = 9;

fn main() {
    let start_time = std::time::SystemTime::now();

    assert!(std::env::args().count() <= 2, "too many arguments");

    let file_name = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "numbers.out".to_string());
    eprintln!("Writing to {}", file_name);

    let out = std::fs::File::create(file_name).expect("while opening file");
    // Buffering makes this max faster.
    let mut out = std::io::BufWriter::with_capacity(1 * 1024 * 1024 /* 1 MB */, out);

    let mut write_num = WriteNumReverse::new(N_DIGITS);
    let count = write_numbers(&mut write_num, &mut out, N_DIGITS);

    out.flush().expect("while flushing file writes");
    eprintln!(
        "Finished, took {} ms",
        start_time.elapsed().expect("elapsed failed").as_millis()
    );
    eprintln!("Total count: {}", count);
}

fn write_numbers<W: WriteNum>(write_num: &mut W, out: &mut impl Write, n_digits: usize) -> usize {
    let max = BASE
        .checked_pow(n_digits as u32)
        .expect("Num type too small")
        - 1;
    assert!(max <= usize::max_value() as Num, "usize cannot address max");

    // BASE^1, BASE^2, ...:  1, 10, 100, 1000
    let powers: Vec<Num> = {
        let mut powers = Vec::with_capacity(n_digits);
        let mut power = 1;
        for _ in 0..n_digits {
            powers.push(power);
            power *= BASE;
        }
        powers
    };

    // A bit set to keep track of all numbers which are too similar to already
    // chosen numbers (e.g. only differ in one digit).
    // We start out with no numbers being too close because we haven't selected
    // any.
    let mut too_similar = Vob::from_elem((max + 1) as usize, false);

    let mut current: Num = 0;
    let mut count: usize = 0;

    // Get the next free number.
    while let Some(next_free) = too_similar.iter_unset_bits(current..).next() {
        count += 1;
        current = next_free as Num;

        write_num.write(out, n_digits, current);

        mark_variations_as_similar(&powers, &mut too_similar, current);
    }
    count
}

fn mark_variations_as_similar(powers: &Vec<Num>, too_similar: &mut Vob, current: Num) {
    // Iterate over all digit positions.
    // digit_factor = 1 for the first digit, 10 for the second, 100 for the third...
    for digit_factor in powers {
        // The current digit at the selected position.
        //
        // Example: current = 3456, _digit_pos = 2, digit_factor = 100
        //          => current_digit = 4
        let current_digit: Num = (current / digit_factor) % BASE;
        // The curent number with the selected digit set to zero.
        //
        // Example:
        //
        // current = 3456, digit_post = 2, digit_factor = 100
        // => current_level = 3056
        let current_level: Num = current - current_digit * digit_factor;
        // Now change the digit in the selected position to all possible
        // values. Note that this includes the original value since this
        // also has to be excluded.
        for change_to_digit in 0..BASE {
            // The current number with the selected digit changed to
            // change_to_digit.
            //
            // Example:
            //
            // exp = 100, current_level = 3056, change_to_digit = 9
            // => with_changed_digit = 3956
            let with_changed_digit: Num = current_level + change_to_digit * digit_factor;
            too_similar.set(with_changed_digit as usize, true);
        }
    }
}

#[test]
fn test_two_digits() {
    let mut write_num = WriteNumEasy::default();
    assert_eq!(
        generate_number_string(&mut write_num, 2),
        numbers_as_string(&mut write_num, 2, &vec![00, 11, 22, 33, 44, 55, 66, 77, 88, 99])
    )
}

#[test]
fn test_three_digits() {
    let mut write_num = WriteNumEasy::default();
    let start = numbers_as_string(
        &mut write_num,
        3,
        &vec![000, 011, 022, 033, 044, 055, 066, 077, 088, 099, 101],
    );
    let generated = generate_number_string(&mut write_num, 3);
    assert!(
        generated.starts_with(&start),
        "Unexpected start: {}",
        generated
            .lines()
            .take(11)
            .flat_map(|s| vec![s, "\n"])
            .collect::<String>()
    );
}

/// Run `write_numbers` for n_digits and return the result as String.
#[cfg(test)]
fn generate_number_string(w: &mut impl WriteNum, n_digits: usize) -> String {
    let mut out = Vec::with_capacity(n_digits * 100);
    write_numbers(w, &mut out, n_digits);
    String::from_utf8_lossy(&out).to_string()
}

/// Write the numbers to a String as `write_numbers` would do.
#[cfg(test)]
fn numbers_as_string(w: &mut impl WriteNum, n_digits: usize, numbers: &[Num]) -> String {
    let mut out = Vec::with_capacity((n_digits+1) * numbers.len() );
    for num in numbers {
        w.write(&mut out, n_digits, *num);
    }
    String::from_utf8_lossy(&out).to_string()
}


trait WriteNum {
    fn write(&mut self, out: &mut impl Write, n_digits: usize, num: Num);
}

#[derive(Default)]
struct WriteNumEasy;

impl WriteNum for WriteNumEasy {
    fn write(&mut self, out: &mut impl Write, n_digits: usize, num: Num) {
        writeln!(out, "{:0width$}", num, width = n_digits).expect("write");
    }
}

#[test]
fn test_write_num_easy() {
    assert_eq!(
        numbers_as_string(&mut WriteNumEasy::default(), 2, &vec![01, 02, 99]),
        "01\n\
         02\n\
         99\n");
}

#[test]
fn test_write_num_reverse() {
    assert_eq!(
        numbers_as_string(&mut WriteNumReverse::new(2), 2, &vec![0, 01, 02, 99]),
        "00\n\
         10\n\
         20\n\
         99\n");
}

struct WriteNumReverse {
    buf: [u8; (N_DIGITS+1)],
}

impl WriteNumReverse {
    fn new(n_digits: usize) -> Self {
        let mut buf = [b'0'; (N_DIGITS+1)];
        buf[n_digits] = b'\n';
        WriteNumReverse {
            buf
        }
    }
}

impl WriteNum for WriteNumReverse {
    fn write(&mut self, out: &mut impl Write, n_digits: usize, mut num: Num) {
        for i in 0..n_digits {
            self.buf[i] = b'0' + (num % BASE) as u8;
            num /= BASE;
        }
        out.write_all(&self.buf[..=n_digits]).expect("while writing");
    }
}
