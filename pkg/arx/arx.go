package arx

import (
	"os"
	"time"

	"github.com/demwunz/arx/internal/journal"
)

// Record creates a new journal entry and returns its ID.
// Creates .arx/journal/ directory if it doesn't exist.
func Record(entry Entry) (string, error) {
	// Validate entry type
	if !isValidType(entry.Type) {
		return "", ErrInvalidType
	}

	// Generate ID
	id, err := journal.GenerateID(journal.EntryType(entry.Type))
	if err != nil {
		return "", err
	}

	// Set date if not provided
	date := entry.Date
	if date.IsZero() {
		date = time.Now()
	}

	// Set default actor if not provided
	actor := entry.Actor
	if actor == "" {
		actor = "human"
	}

	// Create internal entry
	internalEntry := &journal.Entry{
		ID:         id,
		Type:       journal.EntryType(entry.Type),
		Actor:      journal.ActorType(actor),
		Date:       date,
		Title:      entry.Title,
		Scope:      entry.Scope,
		Content:    entry.Body,
		Supersedes: entry.Supersedes,
		ReversedBy: entry.Reverses,
	}

	// Write entry (creates directory if needed)
	if err := journal.Write(internalEntry); err != nil {
		return "", err
	}

	return id, nil
}

// List returns entries with optional filtering.
// Returns empty slice if journal directory doesn't exist.
func List(opts ListOptions) ([]Entry, error) {
	state, err := journal.GetState()
	if err != nil {
		// Check if directory doesn't exist
		if os.IsNotExist(err) {
			return []Entry{}, nil
		}
		return nil, err
	}

	var result []Entry
	for _, ews := range state.Entries {
		entry := convertEntry(&ews.Entry, ews.State)

		// Apply filters
		if opts.Type != "" && entry.Type != opts.Type {
			continue
		}
		if opts.State != "" && entry.State != opts.State {
			continue
		}
		if opts.Scope != "" && entry.Scope != opts.Scope {
			continue
		}

		result = append(result, entry)
	}

	return result, nil
}

// Show returns a single entry by ID.
// Returns ErrNotFound if the entry doesn't exist.
func Show(id string) (*Entry, error) {
	internalEntry, err := journal.ReadByID(id)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, ErrNotFound
		}
		return nil, err
	}

	// Get state to determine entry's derived state
	state, err := journal.GetState()
	if err != nil {
		return nil, err
	}

	entryState := StateActive
	if ews, ok := state.Entries[id]; ok {
		entryState = EntryState(ews.State)
	}

	entry := convertEntry(internalEntry, journal.EntryState(entryState))
	return &entry, nil
}

// convertEntry converts an internal journal entry to the public Entry type.
func convertEntry(e *journal.Entry, state journal.EntryState) Entry {
	return Entry{
		ID:         e.ID,
		Type:       EntryType(e.Type),
		Actor:      string(e.Actor),
		Date:       e.Date,
		Title:      e.Title,
		Scope:      e.Scope,
		Body:       e.Content,
		Supersedes: e.Supersedes,
		Reverses:   e.ReversedBy,
		State:      EntryState(state),
	}
}

// isValidType checks if the entry type is valid.
func isValidType(t EntryType) bool {
	switch t {
	case TypeDecision, TypeAssumption, TypeClarification, TypeBlocker,
		TypeRisk, TypeOverride, TypeDefer, TypeTombstone:
		return true
	}
	return false
}
