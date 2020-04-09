package tests

default eval_struct = false

eval_struct {
    #trace(input)
    input.byte == -1
    input.short == -257
    input.int == -65600
    input.long == -3000000000

    input.ubyte == 1
    input.ushort == 257
    input.uint == 65600
    input.ulong == 3000000000

    #input.float = 1.0499999523162842
    input.double == 2.34

    input.string == "this is a string"

    input.unit == null
    input.unit_struct == null
    input.newtype_struct == 3
    input.struc.a == 1
    input.struc.b == 2

    input.unit_variant == "unit"
    input.newtype_variant.new_type == 64
    input.tuple_variant.tuple = [42, "hello"]
    input.struct_variant.struct.age == 72
    input.struct_variant.struct.msg == "goodbye"

    input["some"] = "there's something here"
    input.none = null

    input.map[1] == 2
    input.map[2] == 3

    input.list == [1, 2, 3]
    input.list[0] == 1
    input.list[1] == 2
    input.list[2] == 3

    input.set | {"b", "a"} == {"a", "b"}
    is_set(input.set)
}
