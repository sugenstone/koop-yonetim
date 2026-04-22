fn main() {
    let sifre = std::env::args().nth(1).unwrap_or_else(|| "admin123".into());
    let hash = bcrypt::hash(&sifre, bcrypt::DEFAULT_COST).unwrap();
    println!("{}", hash);
}
