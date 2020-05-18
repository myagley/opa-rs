package main

import "testing"

func TestRegoNew(t *testing.T) {
	query := "data.example.allow"
	modulename := "example.rego"
	modulecontent := `package example

	default allow = false`

	n, err := RegoNew(query, modulename, modulecontent)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	if n != 1 {
		t.Errorf("first id: got %d, expected %d", n, 1)
	}

	n2, err := RegoNew(query, modulename, modulecontent)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	if n2 != 2 {
		t.Errorf("second id: got %d, expected %d", n2, 2)
	}

	if len(registry) != 2 {
		t.Errorf("registry length: got %d, expected %d", len(registry), 2)
	}

	// Drop one
	RegoDrop(2)

	if len(registry) != 1 {
		t.Errorf("registry length: got %d, expected %d", len(registry), 1)
	}
}

func TestRegoEvalBool_true(t *testing.T) {
	query := "data.example.allow"
	modulename := "example.rego"
	modulecontent := `package example

	default allow = true`

	id, err := RegoNew(query, modulename, modulecontent)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	isdefined, err := RegoEvalBool(id, `{"test": 1, "a": false}`)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	expected := true
	if isdefined != expected {
		t.Errorf("isdefined: got %v, expected %v", isdefined, expected)
	}
}

func TestRegoEvalBool_false(t *testing.T) {
	query := "data.example.allow"
	modulename := "example.rego"
	modulecontent := `package example

	default allow = false`

	id, err := RegoNew(query, modulename, modulecontent)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	isdefined, err := RegoEvalBool(id, `{"test": 1, "a": false}`)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	expected := false
	if isdefined != expected {
		t.Errorf("isdefined: got %v, expected %v", isdefined, expected)
	}
}

func TestRegoEvalBool_undefined(t *testing.T) {
	query := "data.example.allow"
	modulename := "example.rego"
	modulecontent := `package example`

	id, err := RegoNew(query, modulename, modulecontent)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	isdefined, err := RegoEvalBool(id, `{"test": 1, "a": false}`)
	if err != nil {
		t.Errorf("err is not nil: %v", err)
	}

	expected := false
	if isdefined != expected {
		t.Errorf("isdefined: got %v, expected %v", isdefined, expected)
	}
}
