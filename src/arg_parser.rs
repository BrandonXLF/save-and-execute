pub fn string_to_arguments(str: &str) -> Vec<String> {
    let mut args: Vec<String> = vec!["".into()];
    let mut in_quotes = false;
    let mut slashes = 0;

    for char in str.chars() {
        let arg = args.last_mut().unwrap();

        match char {
            '\\' => {
                slashes += 1;
            }
            '"' => {
                arg.push_str(
                    &std::iter::repeat('\\')
                        .take(slashes / 2)
                        .collect::<String>(),
                );

                if slashes % 2 == 1 {
                    arg.push(char);
                } else {
                    in_quotes = !in_quotes;
                }

                slashes = 0;
            }
            ' ' => {
                arg.push_str(&std::iter::repeat('\\').take(slashes).collect::<String>());
                slashes = 0;

                if in_quotes {
                    arg.push(char);
                } else {
                    args.push("".into());
                }
            }
            _ => {
                arg.push_str(&std::iter::repeat('\\').take(slashes).collect::<String>());
                slashes = 0;

                arg.push(char);
            }
        }
    }

    args.last_mut()
        .unwrap()
        .push_str(&std::iter::repeat('\\').take(slashes).collect::<String>());

    return args;
}

pub fn escape_string(str: &str) -> String {
    let has_spaces = str.contains(' ');
    let min_len = if has_spaces { str.len() + 2 } else { str.len() };

    let mut escaped_str: String = String::with_capacity(min_len);
    let mut slashes = 0;

    if has_spaces {
        escaped_str.push('"');
    }

    for char in str.chars() {
        match char {
            '\\' => {
                slashes += 1;
            }
            '"' => {
                escaped_str.push_str(
                    &std::iter::repeat('\\')
                        .take(slashes * 2)
                        .collect::<String>(),
                );
                slashes = 0;

                escaped_str.push('\\');
                escaped_str.push('\"');
            }
            _ => {
                escaped_str.push_str(&std::iter::repeat('\\').take(slashes).collect::<String>());
                slashes = 0;

                escaped_str.push(char);
            }
        }
    }

    if has_spaces {
        escaped_str.push_str(
            &std::iter::repeat('\\')
                .take(slashes * 2)
                .collect::<String>(),
        );
        escaped_str.push('"');
    } else {
        escaped_str.push_str(&std::iter::repeat('\\').take(slashes).collect::<String>());
    }

    return escaped_str;
}
