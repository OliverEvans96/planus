check_type!(Example => value_bool : bool);
check_type!(Example => value_empty_enum : Empty);
check_type!(Example => value_non_empty_enum : NonEmpty);
check_type!(Example => value_uint8 : u8);
check_type!(Example => value_uint16 : u16);
check_type!(Example => value_uint32 : u32);
check_type!(Example => value_uint64 : u64);
check_type!(Example => value_int8 : i8);
check_type!(Example => value_int16 : i16);
check_type!(Example => value_int32 : i32);
check_type!(Example => value_int64 : i64);
check_type!(Example => value_struct : Inner);

check_type!(+['a] ExampleRef<'a> => &self.value_bool() : bool);
check_type!(+['a] ExampleRef<'a> => &self.value_empty_enum() : Result<Empty, planus::errors::UnknownEnumTag>);
check_type!(+['a] ExampleRef<'a> => &self.value_non_empty_enum() : Result<NonEmpty, planus::errors::UnknownEnumTag>);
check_type!(+['a] ExampleRef<'a> => &self.value_uint8() : u8);
check_type!(+['a] ExampleRef<'a> => &self.value_uint16() : u16);
check_type!(+['a] ExampleRef<'a> => &self.value_uint32() : u32);
check_type!(+['a] ExampleRef<'a> => &self.value_uint64() : u64);
check_type!(+['a] ExampleRef<'a> => &self.value_int8() : i8);
check_type!(+['a] ExampleRef<'a> => &self.value_int16() : i16);
check_type!(+['a] ExampleRef<'a> => &self.value_int32() : i32);
check_type!(+['a] ExampleRef<'a> => &self.value_int64() : i64);
check_type!(+['a] ExampleRef<'a> => &self.value_struct() : InnerRef<'a>);
