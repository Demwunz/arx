package main

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/demwunz/arx/internal/checkpoint"
	"github.com/demwunz/arx/internal/journal"
	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
)

func main() {
	s := server.NewMCPServer(
		"arx",
		"1.0.0",
		server.WithToolCapabilities(true),
	)

	// Register tools
	registerAddTool(s)
	registerListTool(s)
	registerShowTool(s)
	registerCheckpointShowTool(s)
	registerCheckpointClearTool(s)
	registerResumeTool(s)

	// Start stdio server
	if err := server.ServeStdio(s); err != nil {
		fmt.Printf("server error: %v\n", err)
	}
}

// AddResult is the response for arx_add
type AddResult struct {
	ID string `json:"id"`
}

// ListEntry represents an entry in the list response
type ListEntry struct {
	ID    string `json:"id"`
	Type  string `json:"type"`
	State string `json:"state"`
	Title string `json:"title"`
	Scope string `json:"scope,omitempty"`
	Date  string `json:"date"`
}

// ListResult is the response for arx_list
type ListResult struct {
	Entries []ListEntry `json:"entries"`
}

// ShowResult is the response for arx_show
type ShowResult struct {
	ID         string `json:"id"`
	Type       string `json:"type"`
	Actor      string `json:"actor"`
	Date       string `json:"date"`
	Title      string `json:"title"`
	State      string `json:"state"`
	Scope      string `json:"scope,omitempty"`
	Content    string `json:"content,omitempty"`
	Supersedes string `json:"supersedes,omitempty"`
	ReversedBy string `json:"reversed_by,omitempty"`
}

// CheckpointResult is the response for arx_checkpoint_show
type CheckpointResult struct {
	Exists       bool   `json:"exists"`
	Version      string `json:"version,omitempty"`
	TaskID       string `json:"task_id,omitempty"`
	Status       string `json:"status,omitempty"`
	StartedAt    string `json:"started_at,omitempty"`
	LastActivity string `json:"last_activity,omitempty"`
	IsStale      bool   `json:"is_stale,omitempty"`
}

// ClearResult is the response for arx_checkpoint_clear
type ClearResult struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

// ResumeResult is the response for arx_resume
type ResumeResult struct {
	Markdown string `json:"markdown"`
}

// getArgs extracts the arguments map from a CallToolRequest
func getArgs(request mcp.CallToolRequest) map[string]any {
	if request.Params.Arguments == nil {
		return make(map[string]any)
	}
	args, ok := request.Params.Arguments.(map[string]any)
	if !ok {
		return make(map[string]any)
	}
	return args
}

func registerAddTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_add",
		mcp.WithDescription("Create a new journal entry"),
		mcp.WithString("type",
			mcp.Required(),
			mcp.Description("Entry type: clarification, decision, override, blocker, assumption, risk, defer, tombstone"),
		),
		mcp.WithString("message",
			mcp.Required(),
			mcp.Description("The entry title/message"),
		),
		mcp.WithString("scope",
			mcp.Description("Optional scope for the entry"),
		),
	)

	s.AddTool(tool, addHandler)
}

func addHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	args := getArgs(request)
	entryType, ok := args["type"].(string)
	if !ok || entryType == "" {
		return mcp.NewToolResultError("type parameter is required"), nil
	}

	message, ok := args["message"].(string)
	if !ok || message == "" {
		return mcp.NewToolResultError("message parameter is required"), nil
	}

	scope, _ := args["scope"].(string)

	// Validate entry type
	if !journal.IsValidEntryType(entryType) {
		validTypes := make([]string, len(journal.ValidEntryTypes))
		for i, t := range journal.ValidEntryTypes {
			validTypes[i] = string(t)
		}
		return mcp.NewToolResultError(fmt.Sprintf("invalid entry type %q; valid types: %s", entryType, strings.Join(validTypes, ", "))), nil
	}

	if strings.TrimSpace(message) == "" {
		return mcp.NewToolResultError("message cannot be empty"), nil
	}

	id, err := journal.GenerateID(journal.EntryType(entryType))
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to generate ID: %v", err)), nil
	}

	entry := &journal.Entry{
		ID:    id,
		Type:  journal.EntryType(entryType),
		Actor: journal.ActorSystem,
		Date:  time.Now(),
		Title: message,
		Scope: scope,
	}

	if err := journal.Write(entry); err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to write entry: %v", err)), nil
	}

	result := AddResult{ID: id}
	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}

func registerListTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_list",
		mcp.WithDescription("List journal entries sorted by date (newest first)"),
		mcp.WithString("type",
			mcp.Description("Filter by entry type"),
		),
		mcp.WithString("state",
			mcp.Description("Filter by state: active, superseded, reversed"),
		),
	)

	s.AddTool(tool, listHandler)
}

func listHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	args := getArgs(request)
	typeFilter, _ := args["type"].(string)
	stateFilter, _ := args["state"].(string)

	state, err := journal.GetState()
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to get journal state: %v", err)), nil
	}

	// Convert map to slice for sorting
	var entries []*journal.EntryWithState
	for _, e := range state.Entries {
		entries = append(entries, e)
	}

	// Sort by date, newest first
	sort.Slice(entries, func(i, j int) bool {
		return entries[i].Date.After(entries[j].Date)
	})

	// Apply filters and build result
	var resultEntries []ListEntry
	for _, e := range entries {
		if typeFilter != "" && string(e.Type) != typeFilter {
			continue
		}
		if stateFilter != "" && string(e.State) != stateFilter {
			continue
		}

		resultEntries = append(resultEntries, ListEntry{
			ID:    e.ID,
			Type:  string(e.Type),
			State: string(e.State),
			Title: e.Title,
			Scope: e.Scope,
			Date:  e.Date.Format(time.RFC3339),
		})
	}

	// Return empty list if no entries (graceful handling per REQ-022)
	if resultEntries == nil {
		resultEntries = []ListEntry{}
	}

	result := ListResult{Entries: resultEntries}
	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}

func registerShowTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_show",
		mcp.WithDescription("Display a single journal entry by ID"),
		mcp.WithString("id",
			mcp.Required(),
			mcp.Description("The entry ID to display"),
		),
	)

	s.AddTool(tool, showHandler)
}

func showHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	args := getArgs(request)
	id, ok := args["id"].(string)
	if !ok || id == "" {
		return mcp.NewToolResultError("id parameter is required"), nil
	}

	entry, err := journal.ReadByID(id)
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to read entry: %v", err)), nil
	}

	// Get state info
	state, err := journal.GetState()
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to get state: %v", err)), nil
	}

	entryState := journal.StateActive
	if es, ok := state.Entries[id]; ok {
		entryState = es.State
	}

	result := ShowResult{
		ID:         entry.ID,
		Type:       string(entry.Type),
		Actor:      string(entry.Actor),
		Date:       entry.Date.Format(time.RFC3339),
		Title:      entry.Title,
		State:      string(entryState),
		Scope:      entry.Scope,
		Content:    entry.Content,
		Supersedes: entry.Supersedes,
		ReversedBy: entry.ReversedBy,
	}

	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}

func registerCheckpointShowTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_checkpoint_show",
		mcp.WithDescription("Display current checkpoint status"),
	)

	s.AddTool(tool, checkpointShowHandler)
}

func checkpointShowHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	cp, err := checkpoint.Load()
	if err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to load checkpoint: %v", err)), nil
	}

	// Handle missing checkpoint gracefully per REQ-023
	if cp == nil {
		result := CheckpointResult{
			Exists: false,
		}
		jsonBytes, _ := json.Marshal(result)
		return mcp.NewToolResultText(string(jsonBytes)), nil
	}

	result := CheckpointResult{
		Exists:       true,
		Version:      cp.Version,
		TaskID:       cp.TaskID,
		Status:       string(cp.Status),
		StartedAt:    cp.StartedAt.Format(time.RFC3339),
		LastActivity: cp.LastActivity.Format(time.RFC3339),
		IsStale:      checkpoint.IsStale(cp),
	}

	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}

func registerCheckpointClearTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_checkpoint_clear",
		mcp.WithDescription("Remove the checkpoint file"),
	)

	s.AddTool(tool, checkpointClearHandler)
}

func checkpointClearHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	if err := checkpoint.Clear(); err != nil {
		return mcp.NewToolResultError(fmt.Sprintf("failed to clear checkpoint: %v", err)), nil
	}

	result := ClearResult{
		Success: true,
		Message: "Checkpoint cleared successfully",
	}
	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}

func registerResumeTool(s *server.MCPServer) {
	tool := mcp.NewTool("arx_resume",
		mcp.WithDescription("Generate resume context as markdown"),
	)

	s.AddTool(tool, resumeHandler)
}

func resumeHandler(ctx context.Context, request mcp.CallToolRequest) (*mcp.CallToolResult, error) {
	var sb strings.Builder
	sb.WriteString("# Resume Context\n\n")
	sb.WriteString(fmt.Sprintf("Generated: %s\n\n", time.Now().Format(time.RFC3339)))

	// Checkpoint status
	sb.WriteString("## Checkpoint\n\n")
	cp, err := checkpoint.Load()
	if err != nil {
		sb.WriteString(fmt.Sprintf("Error loading checkpoint: %v\n\n", err))
	} else if cp == nil {
		sb.WriteString("No active checkpoint.\n\n")
	} else {
		sb.WriteString(fmt.Sprintf("- Task ID: %s\n", cp.TaskID))
		sb.WriteString(fmt.Sprintf("- Status: %s\n", cp.Status))
		sb.WriteString(fmt.Sprintf("- Last Activity: %s\n", cp.LastActivity.Format(time.RFC3339)))
		if checkpoint.IsStale(cp) {
			sb.WriteString("- ⚠️  STALE: Last activity > 72 hours ago\n")
		}
		sb.WriteString("\n")
	}

	// Active journal entries
	sb.WriteString("## Active Journal Entries\n\n")
	state, err := journal.GetState()
	if err != nil {
		sb.WriteString(fmt.Sprintf("Error loading journal: %v\n\n", err))
	} else {
		var activeEntries []*journal.EntryWithState
		for _, e := range state.Entries {
			if e.State == journal.StateActive {
				activeEntries = append(activeEntries, e)
			}
		}

		// Sort by date, newest first
		sort.Slice(activeEntries, func(i, j int) bool {
			return activeEntries[i].Date.After(activeEntries[j].Date)
		})

		if len(activeEntries) == 0 {
			sb.WriteString("No active entries.\n\n")
		} else {
			for _, e := range activeEntries {
				sb.WriteString(fmt.Sprintf("### %s\n", e.ID))
				sb.WriteString(fmt.Sprintf("- Type: %s\n", e.Type))
				sb.WriteString(fmt.Sprintf("- Date: %s\n", e.Date.Format(time.RFC3339)))
				sb.WriteString(fmt.Sprintf("- Title: %s\n", e.Title))
				if e.Scope != "" {
					sb.WriteString(fmt.Sprintf("- Scope: %s\n", e.Scope))
				}
				if e.Content != "" {
					sb.WriteString(fmt.Sprintf("\n%s\n", e.Content))
				}
				sb.WriteString("\n")
			}
		}
	}

	result := ResumeResult{Markdown: sb.String()}
	jsonBytes, _ := json.Marshal(result)
	return mcp.NewToolResultText(string(jsonBytes)), nil
}
