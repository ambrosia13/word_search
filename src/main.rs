fn main() {
    let words = [
        String::from("nap"),
        String::from("sleep"),
        String::from("pillow"),
        String::from("eggplant"),
        String::from("distraction"),
        String::from("sandwich"),
        String::from("anklet"),
        String::from("rats"),
        String::from("skater"),
    ];

    let word_search = word_search::WordSearch::new(&word_search::WordSearchConfig {
        num_rows: 15,
        num_columns: 15,
        words: &words,
        use_only_given_letters_in_grid: false,
        allow_backward_words: true,
    })
    .unwrap();

    println!("{}", word_search);
}
