/// Scans the memory of the current process for a pattern of bytes and returns the
/// address if there is any match.
///
/// `address` is the address to start scanning from.
/// `length` is the maximum amount of bytes to scan from the address.
/// `pattern` is the pattern of bytes to find. None is used to indicate a wildcard
pub unsafe fn pattern_scan(
    address: *const u8,
    length: usize,
    pattern: &[Option<u8>],
) -> Option<*const u8> {
    for address_index in 0..length {
        let addr_offset = address.offset(address_index as isize);
        let has_match = pattern.iter().enumerate().all(|(byte_index, &b)| match b {
            Some(byte) => {
                let addr = addr_offset.offset(byte_index as isize);
                let value = std::ptr::read::<u8>(addr);

                return value == byte;
            }
            None => true,
        });
        if has_match {
            return Some(addr_offset);
        }
    }

    None
    // std::ptr::null_mut() as *const ()
}
