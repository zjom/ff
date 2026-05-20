package tree_sitter_ff_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_ff "github.com/tree-sitter/tree-sitter-ff/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_ff.Language())
	if language == nil {
		t.Errorf("Error loading ff grammar")
	}
}
