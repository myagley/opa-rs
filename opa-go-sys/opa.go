package main

// #include <stdlib.h>
import "C"

import (
	"context"
	"os"
	"unsafe"

	"github.com/open-policy-agent/opa/loader"
	"github.com/open-policy-agent/opa/rego"
)

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

//export Build
func Build(query string, data, bundles, ignore []string) (unsafe.Pointer, int, *C.char) {
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
