package tests

default types = false

types = true {
    is_array(["a string", "another string"])
    is_boolean(true)
    is_null(null)
    is_number(1.23)
    is_object({"key1":"value1", "key2":"value2"})
    is_set({1, 3})
    is_string("a string")

    is_array(array.concat([1], [2]))
    is_boolean(all([true]))
    is_number(1 + 2)
    is_object(object.remove({"key1": "value1"}, ["key1"]))
    is_set({1, 2} & {1, 3})
    is_string(upper("lower"))
}
