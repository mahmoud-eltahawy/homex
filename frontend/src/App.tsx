import { createSignal } from 'solid-js'

function App() {
  const [count, setCount] = createSignal(0)

  return (
    <>
        <button
          class="text-xl"
          onClick={() => setCount((count) => count + 1)}
        >
          count is {count()}
        </button>
    </>
  )
}

export default App
