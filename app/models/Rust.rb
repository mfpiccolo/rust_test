require 'ffi'

module Rust
  extend FFI::Library
  ffi_lib './bin/libembed.dylib'

  class PlusOneNumbers < FFI::Struct
    layout :a, :int,
           :b, :int
  end

  attach_function :parse_page, [:string], :string
end
