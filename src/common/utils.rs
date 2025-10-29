pub fn truncate_namespaces(name: &str, indices: &[u32], max_len: usize) -> (String, Vec<u32>) {
    if name.len() <= max_len {
        return (name.to_string(), indices.to_vec());
    }

    // Replace namespaces with '.' until the name fits within max_len
    let parts: Vec<&str> = name.split('/').skip(1).collect(); // Skip the leading empty part
    let mut namespaces_replaced = 0;
    let mut current_len = name.len();
    for part in parts.iter() {
        if part.is_empty() {
            continue;
        }

        current_len -= part.len() - 1; // +1 for the '.' character
        namespaces_replaced += 1;
        if current_len <= max_len {
            break;
        }
    }

    let truncated_name =
        "/.".repeat(namespaces_replaced) + "/" + &parts[namespaces_replaced..].join("/");

    let new_indices: Vec<u32> = indices
        .iter()
        .filter_map(|&idx| {
            // Length reduction due to replaced namespaces
            let truncated_len = name.len() - current_len + namespaces_replaced * 2;
            if (idx as usize) < truncated_len {
                None
            } else {
                Some(idx - (name.len() - current_len) as u32)
            }
        })
        .collect();
    (truncated_name, new_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_namespaces() {
        let name = "/a/very/long/topic/name/that/needs/truncation";
        let indices = vec![1, 3, 6, 8, 11, 13, 17, 19, 22, 24, 27, 29, 33, 35, 44];
        let max_len = 25;
        let (truncated_name, new_indices) = truncate_namespaces(name, &indices, max_len);
        assert_eq!(truncated_name, "/./././././././truncation");
        assert_eq!(new_indices, vec![15, 24]);

        let max_len = 30;
        let (truncated_name, new_indices) = truncate_namespaces(name, &indices, max_len);
        assert_eq!(truncated_name, "/././././././needs/truncation");
        assert_eq!(new_indices, vec![13, 17, 19, 28]);

        let max_len = 34;
        let (truncated_name, new_indices) = truncate_namespaces(name, &indices, max_len);
        assert_eq!(truncated_name, "/./././././that/needs/truncation");
        assert_eq!(new_indices, vec![11, 14, 16, 20, 22, 31]);
    }
}
