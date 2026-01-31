pub fn jsonc_parse(jsonc: &str) -> String {
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut result = String::new();
    
    // Stack to track braces/brackets for trailing comma detection
    let mut stack = Vec::new();

    let mut chars = jsonc.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                result.push('\n');
            }
            continue;
        } else if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                in_block_comment = false;
                chars.next(); // Skip the '/'
            }
            continue;
        } else if in_string {
            result.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            // Not in string or comment
            match ch {
                '"' => {
                    in_string = true;
                    result.push(ch);
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        in_line_comment = true;
                        chars.next(); // Skip second '/'
                    } else if chars.peek() == Some(&'*') {
                        in_block_comment = true;
                        chars.next(); // Skip '*'
                    } else {
                        result.push(ch);
                    }
                }
                '{' | '[' => {
                    stack.push(ch);
                    result.push(ch);
                }
                '}' | ']' => {
                    // Remove trailing comma before closing brace/bracket
                    let mut temp_result = result.trim_end().to_string();
                    if temp_result.ends_with(',') {
                        temp_result.pop(); // Remove the trailing comma
                    }
                    result = temp_result;
                    result.push(ch);
                    stack.pop();
                }
                ',' => {
                    // Check if this is a trailing comma
                    let mut lookahead = chars.clone();
                    let mut is_trailing = true;
                    
                    while let Some(next_ch) = lookahead.next() {
                        match next_ch {
                            ' ' | '\t' | '\n' | '\r' => continue,
                            '/' => {
                                // Skip comments in lookahead
                                if lookahead.peek() == Some(&'/') {
                                    for c in lookahead.by_ref(){
                                        if c == '\n' { break; }
                                    }
                                    continue;
                                } else if lookahead.peek() == Some(&'*') {
                                    lookahead.next(); // Skip '*'
                                    while let Some(c) = lookahead.next() {
                                        if c == '*' && lookahead.peek() == Some(&'/') {
                                            lookahead.next(); // Skip '/'
                                            break;
                                        }
                                    }
                                    continue;
                                }
                            }
                            '}' | ']' => {
                                is_trailing = true;
                                break;
                            }
                            _ => {
                                is_trailing = false;
                                break;
                            }
                        }
                    }
                    
                    if !is_trailing {
                        result.push(',');
                    }
                    // If it's trailing, we simply skip adding it
                }
                _ => result.push(ch),
            }
        }
    }

    result
}