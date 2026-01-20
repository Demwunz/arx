package arx

// ListOptions configures filtering for List operations.
type ListOptions struct {
	// Type filters entries by entry type.
	Type EntryType

	// State filters entries by derived state.
	State EntryState

	// Scope filters entries by scope.
	Scope string
}

// SearchOptions configures filtering for Search operations.
type SearchOptions struct {
	// Type filters entries by entry type.
	Type EntryType

	// State filters entries by derived state.
	State EntryState

	// Scope filters entries by scope.
	Scope string

	// Limit sets the maximum number of results (0 = no limit).
	Limit int
}
