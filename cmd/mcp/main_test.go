package main

import (
	"context"
	"encoding/json"
	"os"
	"strings"
	"testing"

	"github.com/demwunz/arx/internal/checkpoint"
	"github.com/demwunz/arx/internal/journal"
	"github.com/mark3labs/mcp-go/mcp"
)

func makeRequest(args map[string]any) mcp.CallToolRequest {
	return mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: args,
		},
	}
}

func TestAddHandler_Success(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{
		"type":    "decision",
		"message": "Test decision from MCP",
	})

	result, err := addHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("addHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("addHandler returned error result: %v", result.Content)
	}

	// Parse result
	var addResult AddResult
	textContent, ok := result.Content[0].(mcp.TextContent)
	if !ok {
		t.Fatal("Expected TextContent")
	}
	if err := json.Unmarshal([]byte(textContent.Text), &addResult); err != nil {
		t.Fatalf("Failed to parse result: %v", err)
	}

	if addResult.ID == "" {
		t.Error("Expected non-empty ID")
	}
	if !strings.HasPrefix(addResult.ID, "decision-") {
		t.Errorf("Expected ID to start with 'decision-', got %s", addResult.ID)
	}
}

func TestAddHandler_WithScope(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{
		"type":    "assumption",
		"message": "Test assumption",
		"scope":   "backend",
	})

	result, err := addHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("addHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("addHandler returned error result")
	}

	// Verify scope was saved
	entries, _ := journal.ReadAll()
	if len(entries) != 1 {
		t.Fatalf("Expected 1 entry, got %d", len(entries))
	}
	if entries[0].Scope != "backend" {
		t.Errorf("Scope mismatch: got %q, want %q", entries[0].Scope, "backend")
	}
}

func TestAddHandler_InvalidType(t *testing.T) {
	req := makeRequest(map[string]any{
		"type":    "invalid",
		"message": "Test",
	})

	result, err := addHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("addHandler returned error: %v", err)
	}

	if !result.IsError {
		t.Error("Expected error result for invalid type")
	}
}

func TestAddHandler_MissingType(t *testing.T) {
	req := makeRequest(map[string]any{
		"message": "Test",
	})

	result, err := addHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("addHandler returned error: %v", err)
	}

	if !result.IsError {
		t.Error("Expected error result for missing type")
	}
}

func TestAddHandler_MissingMessage(t *testing.T) {
	req := makeRequest(map[string]any{
		"type": "decision",
	})

	result, err := addHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("addHandler returned error: %v", err)
	}

	if !result.IsError {
		t.Error("Expected error result for missing message")
	}
}

func TestListHandler_Empty(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{})

	result, err := listHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("listHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("listHandler returned error result")
	}

	// Parse result
	var listResult ListResult
	textContent, ok := result.Content[0].(mcp.TextContent)
	if !ok {
		t.Fatal("Expected TextContent")
	}
	if err := json.Unmarshal([]byte(textContent.Text), &listResult); err != nil {
		t.Fatalf("Failed to parse result: %v", err)
	}

	if len(listResult.Entries) != 0 {
		t.Errorf("Expected 0 entries, got %d", len(listResult.Entries))
	}
}

func TestListHandler_WithEntries(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create some entries
	addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "decision",
		"message": "First",
	}))
	addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "assumption",
		"message": "Second",
	}))

	req := makeRequest(map[string]any{})

	result, err := listHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("listHandler returned error: %v", err)
	}

	var listResult ListResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &listResult)

	if len(listResult.Entries) != 2 {
		t.Errorf("Expected 2 entries, got %d", len(listResult.Entries))
	}
}

func TestListHandler_TypeFilter(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "decision",
		"message": "Decision",
	}))
	addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "assumption",
		"message": "Assumption",
	}))

	req := makeRequest(map[string]any{
		"type": "decision",
	})

	result, err := listHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("listHandler returned error: %v", err)
	}

	var listResult ListResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &listResult)

	if len(listResult.Entries) != 1 {
		t.Errorf("Expected 1 entry, got %d", len(listResult.Entries))
	}
	if listResult.Entries[0].Type != "decision" {
		t.Errorf("Expected type 'decision', got %s", listResult.Entries[0].Type)
	}
}

func TestShowHandler_Success(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create an entry
	addResult, _ := addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "decision",
		"message": "Test decision",
	}))
	var add AddResult
	textContent := addResult.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &add)

	// Show it
	req := makeRequest(map[string]any{
		"id": add.ID,
	})

	result, err := showHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("showHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("showHandler returned error result")
	}

	var showResult ShowResult
	textContent = result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &showResult)

	if showResult.ID != add.ID {
		t.Errorf("ID mismatch: got %q, want %q", showResult.ID, add.ID)
	}
	if showResult.Title != "Test decision" {
		t.Errorf("Title mismatch: got %q, want %q", showResult.Title, "Test decision")
	}
	if showResult.State != "active" {
		t.Errorf("State mismatch: got %q, want %q", showResult.State, "active")
	}
}

func TestShowHandler_NotFound(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{
		"id": "nonexistent-id",
	})

	result, err := showHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("showHandler returned error: %v", err)
	}

	if !result.IsError {
		t.Error("Expected error result for nonexistent ID")
	}
}

func TestShowHandler_MissingID(t *testing.T) {
	req := makeRequest(map[string]any{})

	result, err := showHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("showHandler returned error: %v", err)
	}

	if !result.IsError {
		t.Error("Expected error result for missing ID")
	}
}

func TestCheckpointShowHandler_NoCheckpoint(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{})

	result, err := checkpointShowHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("checkpointShowHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("checkpointShowHandler returned error result")
	}

	var cpResult CheckpointResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &cpResult)

	if cpResult.Exists {
		t.Error("Expected Exists to be false")
	}
}

func TestCheckpointShowHandler_WithCheckpoint(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create a checkpoint
	checkpoint.Save(&checkpoint.Checkpoint{
		TaskID: "test-task-123",
	})

	req := makeRequest(map[string]any{})

	result, err := checkpointShowHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("checkpointShowHandler returned error: %v", err)
	}

	var cpResult CheckpointResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &cpResult)

	if !cpResult.Exists {
		t.Error("Expected Exists to be true")
	}
	if cpResult.TaskID != "test-task-123" {
		t.Errorf("TaskID mismatch: got %q, want %q", cpResult.TaskID, "test-task-123")
	}
}

func TestCheckpointClearHandler(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create a checkpoint first
	checkpoint.Save(&checkpoint.Checkpoint{
		TaskID: "test-task",
	})

	req := makeRequest(map[string]any{})

	result, err := checkpointClearHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("checkpointClearHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("checkpointClearHandler returned error result")
	}

	var clearResult ClearResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &clearResult)

	if !clearResult.Success {
		t.Error("Expected Success to be true")
	}

	// Verify checkpoint is gone
	cp, _ := checkpoint.Load()
	if cp != nil {
		t.Error("Checkpoint should be cleared")
	}
}

func TestResumeHandler(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	req := makeRequest(map[string]any{})

	result, err := resumeHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("resumeHandler returned error: %v", err)
	}

	if result.IsError {
		t.Fatalf("resumeHandler returned error result")
	}

	var resumeResult ResumeResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &resumeResult)

	if resumeResult.Markdown == "" {
		t.Error("Expected non-empty markdown")
	}
	if !strings.Contains(resumeResult.Markdown, "# Resume Context") {
		t.Error("Expected markdown to contain '# Resume Context'")
	}
}

func TestResumeHandler_WithData(t *testing.T) {
	os.RemoveAll(".arx")
	defer os.RemoveAll(".arx")

	// Create some data
	addHandler(context.Background(), makeRequest(map[string]any{
		"type":    "decision",
		"message": "Important decision",
	}))
	checkpoint.Save(&checkpoint.Checkpoint{
		TaskID: "resume-test",
	})

	req := makeRequest(map[string]any{})

	result, err := resumeHandler(context.Background(), req)
	if err != nil {
		t.Fatalf("resumeHandler returned error: %v", err)
	}

	var resumeResult ResumeResult
	textContent := result.Content[0].(mcp.TextContent)
	json.Unmarshal([]byte(textContent.Text), &resumeResult)

	if !strings.Contains(resumeResult.Markdown, "resume-test") {
		t.Error("Expected markdown to contain task ID")
	}
	if !strings.Contains(resumeResult.Markdown, "Important decision") {
		t.Error("Expected markdown to contain entry title")
	}
}

func TestGetArgs_NilArguments(t *testing.T) {
	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: nil,
		},
	}

	args := getArgs(req)
	if args == nil {
		t.Error("Expected non-nil map")
	}
	if len(args) != 0 {
		t.Error("Expected empty map")
	}
}

func TestGetArgs_WrongType(t *testing.T) {
	req := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Arguments: "not a map",
		},
	}

	args := getArgs(req)
	if args == nil {
		t.Error("Expected non-nil map")
	}
	if len(args) != 0 {
		t.Error("Expected empty map")
	}
}
