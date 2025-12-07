#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    #[test]
    fn test_basic_iteration_and_pos() {
        let s = "ab\nc";
        let mut lx = Lexer::new(s);

        let (_, ch) = lx.peek().unwrap();
        assert_eq!(ch, 'a');

        // read 'a'
        assert_eq!(lx.next_char(), Some((0, 'a')));
        assert_eq!(lx.pos_index(), 1);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 1);

        // read 'b'
        assert_eq!(lx.next_char(), Some((1, 'b')));
        assert_eq!(lx.pos_index(), 2);
        assert_eq!(lx.line, 0);
        assert_eq!(lx.column, 2);

        // read '\n'
        assert_eq!(lx.next_char(), Some((2, '\n')));
        assert_eq!(lx.pos_index(), 3);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 0);

        // read 'c'
        assert_eq!(lx.next_char(), Some((3, 'c')));
        assert_eq!(lx.pos_index(), 4);
        assert_eq!(lx.line, 1);
        assert_eq!(lx.column, 1);
    }

    #[test]
    fn test_multibyte_chars() {
        // 'é' is multibyte in UTF-8
        let s = "aéb";
        let mut lx = Lexer::new(s);

        let mut got = Vec::new();
        while let Some((_, ch)) = lx.next_char() {
            got.push(ch);
        }
        assert_eq!(got, vec!['a', 'é', 'b']);
        assert_eq!(lx.pos_index(), 3);
    }
}
