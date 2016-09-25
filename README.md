# rushttp
A basic HTTP server parsing/rendering library, written in Rust.

[![Build Status](https://travis-ci.org/thejpster/rushttp.svg?branch=master)](https://travis-ci.org/thejpster/rushttp)

It's not very good - it exists entirely as a learning exercise for me to learn Rust. It's pronounced rush-teep.

Unlike much better libraries, like [hyper](https://github.com/hyperium/hyper), this library is entirely transport agnostic. It parses and emits byte strings, and it is the calling application's responsibility to obtain those /
deliver those to the appropriate TCP socket.
