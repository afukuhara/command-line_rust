fn main() {
    if let Err(e) = cutr::get_args().and_then(cutr::run) {
        eprint!("{}", e);
        std::process::exit(1);
    }
}
