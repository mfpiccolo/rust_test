require 'ffi'

module Rust
  extend FFI::Library
  ffi_lib './bin/libembed.dylib'

  class NodesArray < FFI::Struct
    layout :len,    :size_t, # dynamic array layout
           :data,   :pointer #

    def to_a
      self[:data].get_array_of_string(0, self[:len]).compact
    end
  end

  attach_function :get_links, [:string, :string], NodesArray.by_value
end
