package main

// #include <stdlib.h>
import "C"

import (
	"context"
	"encoding/json"
	"os"
	"sync"
	"unsafe"

	"github.com/open-policy-agent/opa/loader"
	"github.com/open-policy-agent/opa/rego"
)

var (
	registry        = make(map[uint64]*rego.PreparedEvalQuery)
	mutex           = &sync.Mutex{}
	ids      uint64 = 0
)

//export RegoNew
func RegoNew(query string, modulename string, modulecontent string) (uint64, *C.char) {
	ctx := context.Background()

	prepared, err := rego.New(
		rego.Query(query),
		rego.Module(modulename, modulecontent),
	).PrepareForEval(ctx)

	if err != nil {
		return 0, C.CString(err.Error())
	}

	mutex.Lock()
	ids += 1
	var id = ids
	registry[ids] = &prepared
	mutex.Unlock()

	return id, nil
}

//export RegoDrop
func RegoDrop(id uint64) {
	delete(registry, id)
}

//export RegoEvalBool
func RegoEvalBool(id uint64, inputstr string) (bool, *C.char) {
	ctx := context.Background()

	mutex.Lock()
	query, found := registry[id]
	mutex.Unlock()

	if !found {
		return false, C.CString("could not find rego query")
	}

	var input interface{}
	bytes := []byte(inputstr)
	err := json.Unmarshal(bytes, &input)
	if err != nil {
		return false, C.CString(err.Error())
	}

	results, err := query.Eval(ctx, rego.EvalInput(input))
	if err != nil {
		return false, C.CString(err.Error())
	} else if len(results) == 0 {
		return false, nil
	} else if len(results[0].Expressions) > 0 {
		if b, ok := results[0].Expressions[0].Value.(bool); ok {
			return b, nil
		} else {
			return false, nil
		}
	} else {
		return false, nil
	}
}

//export RegoEval
func RegoEval(id uint64, inputstr string) (*C.char, *C.char) {
	ctx := context.Background()

	mutex.Lock()
	query, found := registry[id]
	mutex.Unlock()

	if !found {
		return nil, C.CString("could not find rego query")
	}

	var input interface{}
	bytes := []byte(inputstr)
	err := json.Unmarshal(bytes, &input)
	if err != nil {
		return nil, C.CString(err.Error())
	}

	results, err := query.Eval(ctx, rego.EvalInput(input))
	if err != nil {
		return nil, C.CString(err.Error())
	}

	jbytes, err := json.Marshal(results)
	if err != nil {
		return nil, C.CString(err.Error())
	}

	return C.CString(string(jbytes)), nil
}

// Wasm

type loaderFilter struct {
	Ignore []string
}

func (f loaderFilter) Apply(abspath string, info os.FileInfo, depth int) bool {
	for _, s := range f.Ignore {
		if loader.GlobExcludeName(s, 1)(abspath, info, depth) {
			return true
		}
	}
	return false
}

//export WasmBuild
func WasmBuild(query string, data, bundles, ignore []string) (unsafe.Pointer, int, *C.char) {
	ctx := context.Background()

	f := loaderFilter{
		Ignore: ignore,
	}

	regoArgs := []func(*rego.Rego){
		rego.Query(query),
	}

	if len(data) > 0 {
		regoArgs = append(regoArgs, rego.Load(data, f.Apply))
	}

	if len(bundles) > 0 {
		for _, bundleDir := range bundles {
			regoArgs = append(regoArgs, rego.LoadBundle(bundleDir))
		}
	}

	r := rego.New(regoArgs...)
	cr, err := r.Compile(ctx, rego.CompilePartial(false))
	if err != nil {
		return nil, 0, C.CString(err.Error())
	}

	return C.CBytes(cr.Bytes), len(cr.Bytes), nil
}

//export Free
func Free(ptr unsafe.Pointer) {
	C.free(ptr)
}

func main() {}
