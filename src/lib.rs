use chrono::prelude::*;
use std::fs::*;
use std::io::Read;
use std::io::Write;
use std::ops::Rem;
use std::time::Instant;

const DIGITS_IN_EPOCH_SECOND_TIMESTAMP: usize = 10;
const DIGITS_IN_EPOCH_MILLISECOND_TIMESTAMP: usize = 13;
const NANOS_PER_MILLISECOND: i64 = 1_000_000;
const BUFFER_SIZE: usize = 1024;

pub fn process_files(files: &[String]) {
    let mut options = OpenOptions::new();
    options.read(true);
    let mut buffer = [0; BUFFER_SIZE];
    for file_name in files {
        let target_file_name = file_name.to_string() + &".depoch".to_string();
        let mut file = options.open(file_name).unwrap();
        let mut target_file = options
            .create(true)
            .write(true)
            .open(target_file_name)
            .unwrap();
        match file.read(&mut buffer).ok() {
            Some(read_length) => {
                if read_length != 0 {
                    let replacement = replace_epoch_timestamps_in_buffer(&buffer, read_length, read_length < buffer.len());
                    let slice = replacement.data.as_slice();
                    target_file.write(slice).expect("Failed to write");
                }
            }
            _ => panic!("Error reading from input file"),
        };

        target_file.flush().expect("Error flushing target file")
    }
}

pub struct ReplacementResult {
    pub data: Vec<u8>,
    pub left_over_data: u64,
}

pub fn replace_epoch_timestamps(input: &[u8], end_of_input: bool) -> ReplacementResult {
    replace_epoch_timestamps_in_buffer(input, input.len(), end_of_input)
}

pub fn replace_epoch_timestamps_in_buffer(input: &[u8], input_length: usize, end_of_input: bool) -> ReplacementResult {
    let mut replaced: Vec<u8> = Vec::new();
    let mut integer_accumulator = Vec::new();
    for index in 0..input_length {
        if input[index].is_ascii_digit() {
            integer_accumulator.push(input[index]);
        } else {
            if is_epoch_millisecond_timestamp(&integer_accumulator) {
                append_epoch_timestamp(&mut integer_accumulator, &mut replaced)
            } else if is_epoch_second_timestamp(&integer_accumulator) {
                append_epoch_timestamp(&mut integer_accumulator, &mut replaced)
            }

            replaced.push(input[index]);
        }
    }
    if end_of_input {
        if is_epoch_millisecond_timestamp(&integer_accumulator) {
            append_epoch_timestamp(&mut integer_accumulator, &mut replaced)
        } else if is_epoch_second_timestamp(&integer_accumulator) {
            append_epoch_timestamp(&mut integer_accumulator, &mut replaced)
        }
    }

    if replaced.len() != 0 {
        ReplacementResult {
            data: replaced,
            left_over_data: integer_accumulator.len() as u64,
        }
    } else {
        ReplacementResult {
            data: Vec::from(input),
            left_over_data: integer_accumulator.len() as u64,
        }
    }
}

fn is_epoch_millisecond_timestamp(input: &Vec<u8>) -> bool {
    return input.len() == DIGITS_IN_EPOCH_MILLISECOND_TIMESTAMP;
}

fn is_epoch_second_timestamp(input: &Vec<u8>) -> bool {
    return input.len() == DIGITS_IN_EPOCH_SECOND_TIMESTAMP;
}

fn append_epoch_timestamp(integer_accumulator: &mut Vec<u8>, append_buffer: &mut Vec<u8>) {
    let mut timestamp: i64 = 0;
    let digit_count = integer_accumulator.len();
    integer_accumulator.reverse();
    loop {
        if let Some(next) = integer_accumulator.pop() {
            timestamp *= 10;
            timestamp += (next - 48 as u8) as i64
        } else {
            break;
        }
    }

    let nanos: u32 = match digit_count {
        DIGITS_IN_EPOCH_MILLISECOND_TIMESTAMP => {
            (timestamp.rem(1000) as i64 * NANOS_PER_MILLISECOND) as u32
        }
        DIGITS_IN_EPOCH_SECOND_TIMESTAMP => 0 as u32,
        _ => panic!("Cannot handle {} digits", digit_count),
    };
    let seconds: i64 = match digit_count {
        DIGITS_IN_EPOCH_MILLISECOND_TIMESTAMP => timestamp / 1000,
        DIGITS_IN_EPOCH_SECOND_TIMESTAMP => timestamp,
        _ => panic!("Cannot handle {} digits", digit_count),
    };

    let date_time = Utc.timestamp(seconds, nanos);
    let timestamp_str = format!("[{}]", date_time);
    append_bytes(timestamp_str.as_bytes(), append_buffer);
    integer_accumulator.clear()
}

fn append_bytes(input: &[u8], output: &mut Vec<u8>) {
    for index in 0..input.len() {
        output.push(input[index]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_valid_timestamp_with_millisecond_precision() {
        let input = "1530216070317a";
        let expected = "[2018-06-28 20:01:10.317 UTC]a";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_valid_timestamp_with_second_precision() {
        let input = "1530216070a";
        let expected = "[2018-06-28 20:01:10 UTC]a";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn do_not_replace_millisecond_timestamp_at_end_of_input() {
        let input = "1530216070317";
        let expected = "1530216070317";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(13, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn do_not_replace_second_timestamp_at_end_of_input() {
        let input = "1530216070";
        let expected = "1530216070";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(10, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_millisecond_timestamp_at_end_of_input() {
        let input = "1530216070317";
        let expected = "[2018-06-28 20:01:10.317 UTC]";
        let response = replace_epoch_timestamps(input.as_bytes(), true);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_second_timestamp_at_end_of_input() {
        let input = "1530216070";
        let expected = "[2018-06-28 20:01:10 UTC]";
        let response = replace_epoch_timestamps(input.as_bytes(), true);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_valid_timestamp_with_millisecond_precision_in_place() {
        let input = "prefix1530216070317suffix";
        let expected = "prefix[2018-06-28 20:01:10.317 UTC]suffix";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_valid_timestamp_with_second_precision_in_place() {
        let input = "prefix1530216070suffix";
        let expected = "prefix[2018-06-28 20:01:10 UTC]suffix";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_multiple_timestamp_with_second_precision() {
        let input = "prefix1530216070middle1530216070suffix";
        let expected = "prefix[2018-06-28 20:01:10 UTC]middle[2018-06-28 20:01:10 UTC]suffix";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_multiple_timestamp_with_millisecond_precision() {
        let input = "prefix1530216070317middle1530216070317suffix";
        let expected =
            "prefix[2018-06-28 20:01:10.317 UTC]middle[2018-06-28 20:01:10.317 UTC]suffix";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(0, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn indicate_trailing_numeric_chars() {
        let input = "prefix15302160";
        let expected = "prefix";
        let response = replace_epoch_timestamps(input.as_bytes(), false);

        assert_eq!(8, response.left_over_data);
        compare_bytes(expected.as_bytes(), &response.data);
    }

    #[test]
    fn replace_in_file() {
        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true).truncate(true);
        let timestamp = format!("{:?}", Instant::now());
        let name: String = "/tmp/".to_string() + &timestamp.to_string();
        let name2: String = "/tmp/".to_string() + &timestamp.to_string();
        let mut test_data_file = open_options.open(&name).unwrap();
        let test_data = "abcdef\nsome1530216070timestamp\nfoo\nprefix1530216070317suffix\nbar\n\n";
        let expected = "abcdef\nsome[2018-06-28 20:01:10 UTC]timestamp\nfoo\nprefix[2018-06-28 20:01:10.317 UTC]suffix\nbar\n\n";
        test_data_file.write(test_data.as_bytes()).expect("Failed to write file");
        test_data_file.flush().expect("Failed to flush file");

        process_files(&[name]);
        assert_file_content(name2 + &".depoch".to_string(), expected.as_bytes())
    }

    fn assert_file_content(file_name: String, expected: &[u8]) {
        let mut open_options = OpenOptions::new();
        open_options.read(true);
        let mut input_file = open_options.open(file_name).unwrap();
        let mut buffer = [0; BUFFER_SIZE];
        let _bytes_reader = input_file.read(&mut buffer).unwrap();
        let mut truncated_buffer = Vec::new();
        append_bytes(&buffer, &mut truncated_buffer);
        compare_bytes_len(expected, &truncated_buffer.as_slice(), expected.len());
    }

    fn compare_bytes(a: &[u8], b: &[u8]) {
        assert_eq!(
            a.len(),
            b.len(),
            "Input lengths differ: {}, {}",
            a.len(),
            b.len()
        );

        compare_bytes_len(a, b, b.len())
    }

    fn compare_bytes_len(a: &[u8], b: &[u8], length: usize) {
        for index in 0..length {
            assert_eq!(
                a[index] as char, b[index] as char,
                "Bytes at position {} differ",
                index
            );
        }
    }
}
