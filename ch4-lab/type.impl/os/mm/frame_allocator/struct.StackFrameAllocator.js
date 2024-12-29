(function() {var type_impls = {
"os":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-FrameAllocator-for-StackFrameAllocator\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#58-85\">source</a><a href=\"#impl-FrameAllocator-for-StackFrameAllocator\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"os/mm/frame_allocator/trait.FrameAllocator.html\" title=\"trait os::mm::frame_allocator::FrameAllocator\">FrameAllocator</a> for <a class=\"struct\" href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html\" title=\"struct os::mm::frame_allocator::StackFrameAllocator\">StackFrameAllocator</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#59-65\">source</a><a href=\"#method.new\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.new\" class=\"fn\">new</a>() -&gt; Self</h4></section><section id=\"method.alloc\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#66-75\">source</a><a href=\"#method.alloc\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.alloc\" class=\"fn\">alloc</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>&gt;</h4></section><section id=\"method.dealloc\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#76-84\">source</a><a href=\"#method.dealloc\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.dealloc\" class=\"fn\">dealloc</a>(&amp;mut self, ppn: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>)</h4></section></div></details>","FrameAllocator","os::mm::frame_allocator::FrameAllocatorImpl"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-StackFrameAllocator\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#52-57\">source</a><a href=\"#impl-StackFrameAllocator\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html\" title=\"struct os::mm::frame_allocator::StackFrameAllocator\">StackFrameAllocator</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.init\" class=\"method\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#53-56\">source</a><h4 class=\"code-header\">pub fn <a href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html#tymethod.init\" class=\"fn\">init</a>(&amp;mut self, l: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>, r: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>)</h4></section></div></details>",0,"os::mm::frame_allocator::FrameAllocatorImpl"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()