# XDG Desktop Parser

## String value types

According to the [specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s04.html),
values of types `string` must contain only ASCII characters while
`localestring` and `iconstring` are UTF-8 encoded. Accordingly,
they are all just parsed as native Rust UTF-8 strings.
