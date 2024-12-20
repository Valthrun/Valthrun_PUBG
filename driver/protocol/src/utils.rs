use core::str;

pub fn str_to_fixed_buffer(buffer: &mut [u8], value: &str) -> bool {
    let value = value.as_bytes();
    let copy_length = buffer.len().min(value.len());

    buffer[0..copy_length].copy_from_slice(&value[0..copy_length]);
    if copy_length < buffer.len() {
        /* set the c-string end */
        buffer[copy_length] = 0;
    }

    value.len() <= buffer.len()
}

pub fn fixed_buffer_to_str(buffer: &[u8]) -> Option<&str> {
    let str_length = buffer.iter().position(|v| *v == 0).unwrap_or(buffer.len());
    str::from_utf8(&buffer[0..str_length]).ok()
}
