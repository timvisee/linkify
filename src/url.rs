use std::ops::Range;

use crate::scanner::Scanner;

/// Scan for URLs starting from the trigger character ":", requires "://".
///
/// Based on RFC 3986.
pub struct UrlScanner {
    /// Whether to find URLs with no protocol definition.
    ///
    /// Setting this to `true` allows to find URLs without a protocol definition such as
    /// `https://`, to make links like `example.org` findable. For some URLs the specific protocl
    /// that is used is important, and disabling this need may lead to a lot of false positive
    /// links which should be filtered by the end user.
    /// Please note that this finds URLs not specified in the RFC.
    pub no_proto: bool,
}

impl Scanner for UrlScanner {
    fn scan(&self, s: &str, separator: usize) -> Option<Range<usize>> {
        // let after_slash_slash = colon + 3;
        // // Need at least one character for scheme, and one after '//'
        // // TODO(timvisee): this requires changes?
        // if colon > 0 && after_slash_slash < s.len() && s[colon..].starts_with("://") {
        //     if let Some(start) = self.find_start(&s[0..colon]) {
        //         if let Some(end) = self.find_end(&s[after_slash_slash..]) {
        //             let range = Range {
        //                 start,
        //                 end: after_slash_slash + end,
        //             };
        //             return Some(range);
        //         }
        //     }
        // }
        // None

        // TODO(timvisee): use different terms: colon>separator, and such

        if separator == 0 {
            return None;
        }

        // Detect used separator, being `://` or `.`
        let separator_str = if s[separator..].starts_with("://") {
            "://"
        } else if s[separator..].starts_with('.') {
            "."
        } else {
            return None;
        };

        let after_separator = separator + separator_str.len();

        // Need at least one character for scheme, and one after '//'
        // TODO(timvisee): this requires changes?
        if after_separator < s.len() {
            if let Some(start) = self.find_start(&s[0..separator]) {
                if let Some(end) = self.find_end(&s[after_separator..]) {
                    let range = Range {
                        start,
                        end: after_separator + end,
                    };
                    return Some(range);
                }
            }
        }
        None
    }
}

impl UrlScanner {
    // See "scheme" in RFC 3986
    // TODO(timvisee): this requires changes?
    fn find_start(&self, s: &str) -> Option<usize> {
        let mut first = None;
        let mut digit = None;
        for (i, c) in s.char_indices().rev() {
            match c {
                'a'..='z' | 'A'..='Z' => first = Some(i),
                '0'..='9' => digit = Some(i),
                // scheme special
                // TODO(timvisee): add `:` here if `no_proto` is true?
                ':' | '/' if self.no_proto => digit = Some(i),
                '+' | '-' | '.' => {}
                _ => {
                    break;
                }
            }
        }

        // TODO(timvisee): start cannot have partial protocol with `no_proto` (such as `://`, `//` or `/`)

        // We don't want to extract "abc://foo" out of "1abc://foo".
        // ".abc://foo" and others are ok though, as they feel more like separators.
        if let Some(first) = first {
            if let Some(digit) = digit {
                // Comparing the byte indices with `- 1` is ok as scheme must be ASCII
                if first > 0 && first - 1 == digit {
                    return None;
                }
            }
        }
        first
    }

    fn find_end(&self, s: &str) -> Option<usize> {
        let mut round = 0;
        let mut square = 0;
        let mut curly = 0;
        let mut single_quote = false;

        // TODO(timvisee): should this be false if searching from `.`?
        let mut previous_can_be_last = true;
        let mut end = None;

        for (i, c) in s.char_indices() {
            let can_be_last = match c {
                '\u{00}'..='\u{1F}' | ' ' | '\"' | '<' | '>' | '`' | '\u{7F}'..='\u{9F}' => {
                    // These can never be part of an URL, so stop now. See RFC 3986 and RFC 3987.
                    // Some characters are not in the above list, even they are not in "unreserved"
                    // or "reserved":
                    //   '\\', '^', '{', '|', '}'
                    // The reason for this is that other link detectors also allow them. Also see
                    // below, we require the braces to be balanced.
                    break;
                }
                '?' | '!' | '.' | ',' | ':' | ';' => {
                    // These may be part of an URL but not at the end
                    false
                }
                '/' => {
                    // This may be part of an URL and at the end, but not if the previous character
                    // can't be the end of an URL
                    previous_can_be_last
                }
                '(' => {
                    round += 1;
                    false
                }
                ')' => {
                    round -= 1;
                    if round < 0 {
                        // More closing than opening brackets, stop now
                        break;
                    }
                    true
                }
                '[' => {
                    // Allowed in IPv6 address host
                    square += 1;
                    false
                }
                ']' => {
                    // Allowed in IPv6 address host
                    square -= 1;
                    if square < 0 {
                        // More closing than opening brackets, stop now
                        break;
                    }
                    true
                }
                '{' => {
                    curly += 1;
                    false
                }
                '}' => {
                    curly -= 1;
                    if curly < 0 {
                        // More closing than opening brackets, stop now
                        break;
                    }
                    true
                }
                '\'' => {
                    single_quote = !single_quote;
                    // A single quote can only be the end of an URL if there's an even number
                    !single_quote
                }
                _ => true,
            };
            if can_be_last {
                end = Some(i + c.len_utf8());
            }
            previous_can_be_last = can_be_last;
        }

        end
    }
}
