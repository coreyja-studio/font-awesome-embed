use font_awesome_embed::fa;

fn main() {
    let _: &str = fa!("house", solid, class = "text-xl");
    let _: &str = fa!("house", solid, class = "text-xl text-red-400",);
    let _: &str = fa!("house", solid, family = etch, class = "text-xl");
    let _: &str = fa!("house", solid, class = "text-xl", family = etch);
}
