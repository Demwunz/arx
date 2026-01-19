package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/demwunz/arx/internal/checkpoint"
	"github.com/demwunz/arx/internal/journal"
)

const resumePromptPath = ".arx/resume-prompt.md"

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	switch os.Args[1] {
	case "add":
		if err := cmdAdd(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "list":
		if err := cmdList(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "show":
		if err := cmdShow(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "checkpoint":
		if err := cmdCheckpoint(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "resume":
		if err := cmdResume(os.Args[2:]); err != nil {
			fmt.Fprintf(os.Stderr, "error: %v\n", err)
			os.Exit(1)
		}
	case "--help", "-h", "help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", os.Args[1])
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println(`arx - Journal and checkpoint management

Usage:
  arx <command> [arguments]

Commands:
  add <type> "<message>"  Add a new journal entry
      --scope             Entry scope (optional)
      --supersedes        ID of entry this supersedes (optional)
      --reverses          ID of entry this reverses (optional)

  list                    List all journal entries
      --type              Filter by entry type
      --state             Filter by state (active)

  show <id>               Show a single journal entry

  checkpoint [subcommand] Manage checkpoint state
      show                Display current checkpoint (default)
      clear               Remove checkpoint file

  resume                  Generate resume context
      --print             Print to stdout instead of file

  --help                  Show this help message`)
}

func cmdAdd(args []string) error {
	fs := flag.NewFlagSet("add", flag.ContinueOnError)
	scope := fs.String("scope", "", "entry scope")
	supersedes := fs.String("supersedes", "", "ID of entry this supersedes")
	reverses := fs.String("reverses", "", "ID of entry this reverses")

	// Check for unknown flags before parsing
	for _, arg := range args {
		if strings.HasPrefix(arg, "-") && !strings.HasPrefix(arg, "--scope") &&
			!strings.HasPrefix(arg, "--supersedes") && !strings.HasPrefix(arg, "--reverses") &&
			arg != "--scope" && arg != "--supersedes" && arg != "--reverses" {
			if strings.HasPrefix(arg, "--") {
				return fmt.Errorf("unknown flag: %s", arg)
			}
		}
	}

	if err := fs.Parse(args); err != nil {
		return err
	}

	remaining := fs.Args()
	if len(remaining) < 2 {
		return fmt.Errorf("usage: arx add <type> \"<message>\"")
	}

	entryType := remaining[0]
	message := strings.Join(remaining[1:], " ")

	if !journal.IsValidEntryType(entryType) {
		validTypes := make([]string, len(journal.ValidEntryTypes))
		for i, t := range journal.ValidEntryTypes {
			validTypes[i] = string(t)
		}
		return fmt.Errorf("invalid entry type %q; valid types: %s", entryType, strings.Join(validTypes, ", "))
	}

	if strings.TrimSpace(message) == "" {
		return fmt.Errorf("message cannot be empty")
	}

	id, err := journal.GenerateID(journal.EntryType(entryType))
	if err != nil {
		return fmt.Errorf("failed to generate ID: %w", err)
	}

	entry := &journal.Entry{
		ID:         id,
		Type:       journal.EntryType(entryType),
		Actor:      journal.ActorHuman,
		Date:       time.Now(),
		Title:      message,
		Scope:      *scope,
		Supersedes: *supersedes,
	}

	if err := journal.Write(entry); err != nil {
		return fmt.Errorf("failed to write entry: %w", err)
	}

	// If reverses is set, update the target entry
	if *reverses != "" {
		if err := journal.UpdateReversedBy(*reverses, id); err != nil {
			return fmt.Errorf("failed to update reversed entry: %w", err)
		}
	}

	fmt.Printf("Created entry: %s\n", id)
	return nil
}

func cmdList(args []string) error {
	fs := flag.NewFlagSet("list", flag.ContinueOnError)
	typeFilter := fs.String("type", "", "filter by entry type")
	stateFilter := fs.String("state", "", "filter by state (active)")

	if err := fs.Parse(args); err != nil {
		return err
	}

	state, err := journal.GetState()
	if err != nil {
		return fmt.Errorf("failed to get journal state: %w", err)
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

	// Apply filters
	for _, e := range entries {
		if *typeFilter != "" && string(e.Type) != *typeFilter {
			continue
		}
		if *stateFilter != "" && string(e.State) != *stateFilter {
			continue
		}

		fmt.Printf("[%s] %s (%s) - %s\n", e.State, e.ID, e.Type, e.Title)
	}

	return nil
}

func cmdShow(args []string) error {
	if len(args) < 1 {
		return fmt.Errorf("usage: arx show <id>")
	}

	id := args[0]

	entry, err := journal.ReadByID(id)
	if err != nil {
		return fmt.Errorf("failed to read entry: %w", err)
	}

	// Get state info
	state, err := journal.GetState()
	if err != nil {
		return fmt.Errorf("failed to get state: %w", err)
	}

	entryState := journal.StateActive
	if es, ok := state.Entries[id]; ok {
		entryState = es.State
	}

	fmt.Printf("ID:          %s\n", entry.ID)
	fmt.Printf("Type:        %s\n", entry.Type)
	fmt.Printf("Actor:       %s\n", entry.Actor)
	fmt.Printf("Date:        %s\n", entry.Date.Format(time.RFC3339))
	fmt.Printf("Title:       %s\n", entry.Title)
	fmt.Printf("State:       %s\n", entryState)
	if entry.Scope != "" {
		fmt.Printf("Scope:       %s\n", entry.Scope)
	}
	if entry.Supersedes != "" {
		fmt.Printf("Supersedes:  %s\n", entry.Supersedes)
	}
	if entry.ReversedBy != "" {
		fmt.Printf("Reversed By: %s\n", entry.ReversedBy)
	}
	if entry.Content != "" {
		fmt.Printf("\nContent:\n%s\n", entry.Content)
	}

	return nil
}

func cmdCheckpoint(args []string) error {
	subcmd := "show"
	if len(args) > 0 {
		subcmd = args[0]
	}

	switch subcmd {
	case "show":
		cp, err := checkpoint.Load()
		if err != nil {
			return fmt.Errorf("failed to load checkpoint: %w", err)
		}
		if cp == nil {
			fmt.Println("No checkpoint found.")
			return nil
		}

		fmt.Printf("Version:       %s\n", cp.Version)
		fmt.Printf("Task ID:       %s\n", cp.TaskID)
		fmt.Printf("Status:        %s\n", cp.Status)
		fmt.Printf("Started At:    %s\n", cp.StartedAt.Format(time.RFC3339))
		fmt.Printf("Last Activity: %s\n", cp.LastActivity.Format(time.RFC3339))

		if checkpoint.IsStale(cp) {
			fmt.Println("\n⚠️  WARNING: Checkpoint is stale (last activity > 72 hours ago)")
		}

	case "clear":
		if err := checkpoint.Clear(); err != nil {
			return fmt.Errorf("failed to clear checkpoint: %w", err)
		}
		fmt.Println("Checkpoint cleared.")

	default:
		return fmt.Errorf("unknown subcommand: %s; use 'show' or 'clear'", subcmd)
	}

	return nil
}

func cmdResume(args []string) error {
	fs := flag.NewFlagSet("resume", flag.ContinueOnError)
	printFlag := fs.Bool("print", false, "print to stdout instead of file")

	if err := fs.Parse(args); err != nil {
		return err
	}

	// Build resume context
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

	content := sb.String()

	if *printFlag {
		fmt.Print(content)
		return nil
	}

	// Write to file
	dir := filepath.Dir(resumePromptPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	if err := os.WriteFile(resumePromptPath, []byte(content), 0644); err != nil {
		return fmt.Errorf("failed to write resume file: %w", err)
	}

	fmt.Printf("Resume context written to %s\n", resumePromptPath)
	return nil
}
