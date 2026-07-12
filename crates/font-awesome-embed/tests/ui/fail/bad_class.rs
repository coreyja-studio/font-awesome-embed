use font_awesome_embed::fa;

fn main() {
    let _ = fa!("house", solid, class = "foo\" onload=\"evil()");
    let _ = fa!("house", solid, class = "");
    let _ = fa!("house", solid, class = "a", class = "b");
}
