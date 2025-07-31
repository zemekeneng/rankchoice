# Svelte 5 Syntax Guide for RankChoice.app

This guide covers the Svelte 5 runes and patterns used throughout the RankChoice.app project.

## Key Svelte 5 Concepts

### 1. State Management with Runes

#### `$state` - Reactive State
```typescript
// Old Svelte 4 way
let count = 0;

// Svelte 5 way
let count = $state(0);
```

#### `$derived` - Computed Values
```typescript
// Old Svelte 4 way
$: doubled = count * 2;

// Svelte 5 way
let doubled = $derived(count * 2);
```

#### `$effect` - Side Effects
```typescript
// Old Svelte 4 way
$: {
  console.log(`Count is now ${count}`);
  saveToLocalStorage(count);
}

// Svelte 5 way
$effect(() => {
  console.log(`Count is now ${count}`);
  saveToLocalStorage(count);
});
```

### 2. Component Props

#### Basic Props
```typescript
// Old Svelte 4 way
export let name: string;
export let age: number = 25;

// Svelte 5 way
interface Props {
  name: string;
  age?: number;
}

let { name, age = 25 }: Props = $props();
```

#### Bindable Props
```typescript
// Parent component
<ChildComponent bind:value={myValue} />

// Child component (Svelte 5)
interface Props {
  value: string;
}

let { value = $bindable() }: Props = $props();
```

### 3. Stores in Svelte 5

Create stores using classes with runes:

```typescript
// stores/counter.svelte.ts
class CounterStore {
  count = $state(0);
  doubled = $derived(this.count * 2);

  increment() {
    this.count++;
  }

  reset() {
    this.count = 0;
  }
}

export const counter = new CounterStore();
```

Using the store in a component:
```svelte
<script lang="ts">
  import { counter } from '$lib/stores/counter.svelte';
</script>

<button onclick={() => counter.increment()}>
  Count: {counter.count} (Doubled: {counter.doubled})
</button>
```

## RankChoice.app Specific Examples

### 1. Form Handling with Svelte 5

```svelte
<script lang="ts">
  interface FormData {
    email: string;
    password: string;
  }

  let formData = $state<FormData>({
    email: '',
    password: ''
  });

  let errors = $state<Partial<FormData>>({});
  let isSubmitting = $state(false);

  // Derived state for validation
  let isValid = $derived(
    formData.email.includes('@') && 
    formData.password.length >= 8
  );

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!isValid) return;

    isSubmitting = true;
    try {
      // Submit logic
    } finally {
      isSubmitting = false;
    }
  }
</script>

<form onsubmit={handleSubmit}>
  <input 
    type="email" 
    bind:value={formData.email}
    placeholder="Email"
  />
  <input 
    type="password" 
    bind:value={formData.password}
    placeholder="Password"
  />
  <button disabled={!isValid || isSubmitting}>
    {isSubmitting ? 'Submitting...' : 'Submit'}
  </button>
</form>
```

### 2. List Management

```svelte
<script lang="ts">
  import type { Candidate } from '$lib/types';

  interface Props {
    initialCandidates: Candidate[];
  }

  let { initialCandidates }: Props = $props();
  
  let candidates = $state([...initialCandidates]);
  let filter = $state('');

  let filteredCandidates = $derived(
    candidates.filter(c => 
      c.name.toLowerCase().includes(filter.toLowerCase())
    )
  );

  function addCandidate(name: string) {
    candidates = [...candidates, {
      id: crypto.randomUUID(),
      name,
      displayOrder: candidates.length
    }];
  }

  function removeCandidate(id: string) {
    candidates = candidates.filter(c => c.id !== id);
  }
</script>
```

### 3. API Integration with Effects

```svelte
<script lang="ts">
  import { api } from '$lib/api';
  import type { Poll } from '$lib/types';

  interface Props {
    pollId: string;
  }

  let { pollId }: Props = $props();
  
  let poll = $state<Poll | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Load poll data when component mounts or pollId changes
  $effect(() => {
    loading = true;
    error = null;
    
    api.polls.get(pollId)
      .then(data => {
        poll = data;
      })
      .catch(err => {
        error = err.message;
      })
      .finally(() => {
        loading = false;
      });
  });

  // Auto-refresh every 30 seconds
  $effect(() => {
    const interval = setInterval(() => {
      if (poll && !loading) {
        api.polls.get(pollId).then(data => {
          poll = data;
        });
      }
    }, 30000);

    return () => clearInterval(interval);
  });
</script>
```

### 4. Advanced Store Pattern

```typescript
// stores/polls.svelte.ts
import type { Poll } from '$lib/types';
import { api } from '$lib/api';

class PollsStore {
  polls = $state<Poll[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  
  // Derived states
  activePolls = $derived(
    this.polls.filter(p => new Date(p.closesAt) > new Date())
  );
  
  totalVotes = $derived(
    this.polls.reduce((sum, poll) => sum + (poll.voteCount || 0), 0)
  );

  async loadPolls() {
    this.loading = true;
    this.error = null;
    
    try {
      const data = await api.polls.list();
      this.polls = data;
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Failed to load polls';
    } finally {
      this.loading = false;
    }
  }

  async createPoll(data: CreatePollForm) {
    try {
      const newPoll = await api.polls.create(data);
      this.polls = [...this.polls, newPoll];
      return newPoll;
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Failed to create poll';
      throw err;
    }
  }

  updatePoll(id: string, updates: Partial<Poll>) {
    this.polls = this.polls.map(poll => 
      poll.id === id ? { ...poll, ...updates } : poll
    );
  }
}

export const pollsStore = new PollsStore();
```

### 5. Component Composition

```svelte
<!-- Parent.svelte -->
<script lang="ts">
  import Child from './Child.svelte';
  
  let sharedValue = $state('');
  let items = $state<string[]>([]);

  function handleItemAdded(item: string) {
    items = [...items, item];
  }
</script>

<Child 
  bind:value={sharedValue} 
  onitemadded={handleItemAdded}
/>

<!-- Child.svelte -->
<script lang="ts">
  interface Props {
    value: string;
    onitemadded?: (item: string) => void;
  }

  let { 
    value = $bindable(), 
    onitemadded 
  }: Props = $props();

  function addItem() {
    if (value && onitemadded) {
      onitemadded(value);
      value = ''; // This updates the parent's sharedValue
    }
  }
</script>
```

## Migration Tips

### From Svelte 4 to Svelte 5

1. **Replace `let` with `$state()` for reactive variables**
   ```typescript
   // Before
   let count = 0;
   
   // After
   let count = $state(0);
   ```

2. **Replace `$:` with `$derived()` or `$effect()`**
   ```typescript
   // Before
   $: doubled = count * 2;
   $: console.log(count);
   
   // After
   let doubled = $derived(count * 2);
   $effect(() => console.log(count));
   ```

3. **Replace `export let` with `$props()`**
   ```typescript
   // Before
   export let name: string;
   export let age = 25;
   
   // After
   interface Props {
     name: string;
     age?: number;
   }
   let { name, age = 25 }: Props = $props();
   ```

4. **Update stores to use classes with runes**
   ```typescript
   // Before (writable store)
   import { writable } from 'svelte/store';
   export const count = writable(0);
   
   // After
   class CountStore {
     value = $state(0);
   }
   export const count = new CountStore();
   ```

## Best Practices

1. **Always type your props interfaces**
2. **Use `$derived` for computed values instead of calculating in templates**
3. **Clean up effects that create subscriptions or timers**
4. **Prefer `$state` over `$bindable` unless two-way binding is necessary**
5. **Keep stores in `.svelte.ts` files for proper type inference**
6. **Use meaningful variable names for state (not just `state`)**

## Common Patterns

### Loading States
```typescript
let data = $state<T | null>(null);
let loading = $state(true);
let error = $state<Error | null>(null);
```

### Form Handling
```typescript
let form = $state({ field1: '', field2: '' });
let errors = $state<Partial<typeof form>>({});
let touched = $state<Partial<Record<keyof typeof form, boolean>>>({});
```

### Pagination
```typescript
let page = $state(1);
let pageSize = $state(20);
let total = $state(0);
let pages = $derived(Math.ceil(total / pageSize));
let hasNext = $derived(page < pages);
let hasPrev = $derived(page > 1);
```

## Resources

- [Svelte 5 Documentation](https://svelte-5-preview.vercel.app/)
- [Svelte 5 Migration Guide](https://svelte-5-preview.vercel.app/docs/migration-guide)
- [Runes API Reference](https://svelte-5-preview.vercel.app/docs/runes) 