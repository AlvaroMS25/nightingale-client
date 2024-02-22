fn main() {
    #[cfg(not(any(feature = "twilight", feature = "serenity")))]
    compile_error!("Select one of `twilight` or `serenity` features");

    #[cfg(all(feature = "serenity", feature = "twilight"))]
    compile_error!("Can't enable both `twilight` and `serenity` features at the same time");
}