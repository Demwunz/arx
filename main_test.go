package main

import (
	"os"
	"strings"
	"testing"

	"github.com/demwunz/arx/internal/checkpoint"
	"github.com/demwunz/arx/internal/journal"
)

func TestCmdAdd_Success(t *testing.T) {
	// Clean up before and after test
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdAdd([]string{"decision", "Test decision message"})
	if err != nil {
		t.Fatalf("cmdAdd failed: %v", err)
	}

	// Verify entry was created
	entries, err := journal.ReadAll()
	if err != nil {
		t.Fatalf("Failed to read entries: %v", err)
	}

	if len(entries) != 1 {
		t.Fatalf("Expected 1 entry, got %d", len(entries))
	}

	if entries[0].Title != "Test decision message" {
		t.Errorf("Title mismatch: got %q, want %q", entries[0].Title, "Test decision message")
	}
	if entries[0].Type != journal.EntryTypeDecision {
		t.Errorf("Type mismatch: got %q, want %q", entries[0].Type, journal.EntryTypeDecision)
	}
}

func TestCmdAdd_WithScope(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdAdd([]string{"--scope", "backend", "assumption", "API will use REST"})
	if err != nil {
		t.Fatalf("cmdAdd failed: %v", err)
	}

	entries, err := journal.ReadAll()
	if err != nil {
		t.Fatalf("Failed to read entries: %v", err)
	}

	if len(entries) != 1 {
		t.Fatalf("Expected 1 entry, got %d", len(entries))
	}

	if entries[0].Scope != "backend" {
		t.Errorf("Scope mismatch: got %q, want %q", entries[0].Scope, "backend")
	}
}

func TestCmdAdd_InvalidType(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdAdd([]string{"invalid", "Some message"})
	if err == nil {
		t.Fatal("Expected error for invalid type, got nil")
	}

	if !strings.Contains(err.Error(), "invalid entry type") {
		t.Errorf("Expected 'invalid entry type' error, got: %v", err)
	}
}

func TestCmdAdd_EmptyMessage(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdAdd([]string{"decision", "   "})
	if err == nil {
		t.Fatal("Expected error for empty message, got nil")
	}

	if !strings.Contains(err.Error(), "message cannot be empty") {
		t.Errorf("Expected 'message cannot be empty' error, got: %v", err)
	}
}

func TestCmdAdd_MissingArgs(t *testing.T) {
	err := cmdAdd([]string{"decision"})
	if err == nil {
		t.Fatal("Expected error for missing args, got nil")
	}

	if !strings.Contains(err.Error(), "usage:") {
		t.Errorf("Expected usage error, got: %v", err)
	}
}

func TestCmdAdd_Supersedes(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create first entry
	err := cmdAdd([]string{"decision", "Original decision"})
	if err != nil {
		t.Fatalf("First cmdAdd failed: %v", err)
	}

	entries, _ := journal.ReadAll()
	firstID := entries[0].ID

	// Create superseding entry
	err = cmdAdd([]string{"--supersedes", firstID, "decision", "Updated decision"})
	if err != nil {
		t.Fatalf("Second cmdAdd failed: %v", err)
	}

	entries, _ = journal.ReadAll()
	if len(entries) != 2 {
		t.Fatalf("Expected 2 entries, got %d", len(entries))
	}

	// Find the new entry and check supersedes
	for _, e := range entries {
		if e.Title == "Updated decision" {
			if e.Supersedes != firstID {
				t.Errorf("Supersedes mismatch: got %q, want %q", e.Supersedes, firstID)
			}
		}
	}
}

func TestCmdList_Empty(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Should not error even with no entries
	err := cmdList([]string{})
	if err != nil {
		t.Fatalf("cmdList failed: %v", err)
	}
}

func TestCmdList_WithEntries(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create some entries
	cmdAdd([]string{"decision", "First decision"})
	cmdAdd([]string{"assumption", "First assumption"})

	err := cmdList([]string{})
	if err != nil {
		t.Fatalf("cmdList failed: %v", err)
	}
}

func TestCmdList_TypeFilter(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	cmdAdd([]string{"decision", "A decision"})
	cmdAdd([]string{"assumption", "An assumption"})

	// Should not error
	err := cmdList([]string{"--type", "decision"})
	if err != nil {
		t.Fatalf("cmdList with type filter failed: %v", err)
	}
}

func TestCmdShow_NotFound(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdShow([]string{"nonexistent-id"})
	if err == nil {
		t.Fatal("Expected error for nonexistent ID, got nil")
	}
}

func TestCmdShow_MissingID(t *testing.T) {
	err := cmdShow([]string{})
	if err == nil {
		t.Fatal("Expected error for missing ID, got nil")
	}

	if !strings.Contains(err.Error(), "usage:") {
		t.Errorf("Expected usage error, got: %v", err)
	}
}

func TestCmdShow_Success(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	cmdAdd([]string{"decision", "Test decision"})

	entries, _ := journal.ReadAll()
	if len(entries) == 0 {
		t.Fatal("No entries created")
	}

	err := cmdShow([]string{entries[0].ID})
	if err != nil {
		t.Fatalf("cmdShow failed: %v", err)
	}
}

func TestCmdCheckpoint_Show_NoCheckpoint(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdCheckpoint([]string{"show"})
	if err != nil {
		t.Fatalf("cmdCheckpoint show failed: %v", err)
	}
}

func TestCmdCheckpoint_Show_Default(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// "show" is default subcommand
	err := cmdCheckpoint([]string{})
	if err != nil {
		t.Fatalf("cmdCheckpoint (default show) failed: %v", err)
	}
}

func TestCmdCheckpoint_Clear(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create a checkpoint first
	cp := &checkpoint.Checkpoint{
		TaskID: "test-task",
	}
	checkpoint.Save(cp)

	err := cmdCheckpoint([]string{"clear"})
	if err != nil {
		t.Fatalf("cmdCheckpoint clear failed: %v", err)
	}

	// Verify it's gone
	loaded, _ := checkpoint.Load()
	if loaded != nil {
		t.Error("Checkpoint should be cleared")
	}
}

func TestCmdCheckpoint_InvalidSubcommand(t *testing.T) {
	err := cmdCheckpoint([]string{"invalid"})
	if err == nil {
		t.Fatal("Expected error for invalid subcommand, got nil")
	}

	if !strings.Contains(err.Error(), "unknown subcommand") {
		t.Errorf("Expected 'unknown subcommand' error, got: %v", err)
	}
}

func TestCmdResume_Print(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdResume([]string{"--print"})
	if err != nil {
		t.Fatalf("cmdResume --print failed: %v", err)
	}
}

func TestCmdResume_WriteFile(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	err := cmdResume([]string{})
	if err != nil {
		t.Fatalf("cmdResume failed: %v", err)
	}

	// Verify file was created
	if _, err := os.Stat(resumePromptPath); os.IsNotExist(err) {
		t.Error("Resume prompt file was not created")
	}
}

func TestCmdResume_WithData(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create some data
	cmdAdd([]string{"decision", "Test decision"})
	checkpoint.Save(&checkpoint.Checkpoint{
		TaskID: "test-task",
	})

	err := cmdResume([]string{"--print"})
	if err != nil {
		t.Fatalf("cmdResume with data failed: %v", err)
	}
}

func TestAllValidEntryTypes(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	validTypes := []string{
		"clarification",
		"decision",
		"override",
		"blocker",
		"assumption",
		"risk",
		"defer",
		"tombstone",
	}

	for _, entryType := range validTypes {
		t.Run(entryType, func(t *testing.T) {
			err := cmdAdd([]string{entryType, "Test " + entryType})
			if err != nil {
				t.Errorf("cmdAdd failed for type %s: %v", entryType, err)
			}
		})
	}
}
