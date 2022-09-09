#include <array>
#include <cstddef>
#include <cstdint>
#include <new>
#include <string>
#include <type_traits>
#include <utility>

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

namespace {
template <typename T>
class impl;
} // namespace

class String;

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) &noexcept = default;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_BOX
#define CXXBRIDGE1_RUST_BOX
template <typename T>
class Box final {
public:
  using element_type = T;
  using const_pointer =
      typename std::add_pointer<typename std::add_const<T>::type>::type;
  using pointer = typename std::add_pointer<T>::type;

  Box() = delete;
  Box(Box &&) noexcept;
  ~Box() noexcept;

  explicit Box(const T &);
  explicit Box(T &&);

  Box &operator=(Box &&) &noexcept;

  const T *operator->() const noexcept;
  const T &operator*() const noexcept;
  T *operator->() noexcept;
  T &operator*() noexcept;

  template <typename... Fields>
  static Box in_place(Fields &&...);

  void swap(Box &) noexcept;

  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

  /* Deprecated */ using value_type = element_type;

private:
  class uninit;
  class allocation;
  Box(uninit) noexcept;
  void drop() noexcept;

  friend void swap(Box &lhs, Box &rhs) noexcept { lhs.swap(rhs); }

  T *ptr;
};

template <typename T>
class Box<T>::uninit {};

template <typename T>
class Box<T>::allocation {
  static T *alloc() noexcept;
  static void dealloc(T *) noexcept;

public:
  allocation() noexcept : ptr(alloc()) {}
  ~allocation() noexcept {
    if (this->ptr) {
      dealloc(this->ptr);
    }
  }
  T *ptr;
};

template <typename T>
Box<T>::Box(Box &&other) noexcept : ptr(other.ptr) {
  other.ptr = nullptr;
}

template <typename T>
Box<T>::Box(const T &val) {
  allocation alloc;
  ::new (alloc.ptr) T(val);
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::Box(T &&val) {
  allocation alloc;
  ::new (alloc.ptr) T(std::move(val));
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::~Box() noexcept {
  if (this->ptr) {
    this->drop();
  }
}

template <typename T>
Box<T> &Box<T>::operator=(Box &&other) &noexcept {
  if (this->ptr) {
    this->drop();
  }
  this->ptr = other.ptr;
  other.ptr = nullptr;
  return *this;
}

template <typename T>
const T *Box<T>::operator->() const noexcept {
  return this->ptr;
}

template <typename T>
const T &Box<T>::operator*() const noexcept {
  return *this->ptr;
}

template <typename T>
T *Box<T>::operator->() noexcept {
  return this->ptr;
}

template <typename T>
T &Box<T>::operator*() noexcept {
  return *this->ptr;
}

template <typename T>
template <typename... Fields>
Box<T> Box<T>::in_place(Fields &&...fields) {
  allocation alloc;
  auto ptr = alloc.ptr;
  ::new (ptr) T{std::forward<Fields>(fields)...};
  alloc.ptr = nullptr;
  return from_raw(ptr);
}

template <typename T>
void Box<T>::swap(Box &rhs) noexcept {
  using std::swap;
  swap(this->ptr, rhs.ptr);
}

template <typename T>
Box<T> Box<T>::from_raw(T *raw) noexcept {
  Box box = uninit{};
  box.ptr = raw;
  return box;
}

template <typename T>
T *Box<T>::into_raw() noexcept {
  T *raw = this->ptr;
  this->ptr = nullptr;
  return raw;
}

template <typename T>
Box<T>::Box(uninit) noexcept {}
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_OPAQUE
#define CXXBRIDGE1_RUST_OPAQUE
class Opaque {
public:
  Opaque() = delete;
  Opaque(const Opaque &) = delete;
  ~Opaque() = delete;
};
#endif // CXXBRIDGE1_RUST_OPAQUE

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT
} // namespace cxxbridge1
} // namespace rust

struct SimpleDDR4;

#ifndef CXXBRIDGE1_STRUCT_SimpleDDR4
#define CXXBRIDGE1_STRUCT_SimpleDDR4
struct SimpleDDR4 final : public ::rust::Opaque {
  void tick_ddr4() noexcept;
  bool try_send_addr(::std::uint64_t addr, bool is_write) noexcept;
  bool try_recv_addr(::std::uint64_t &addr, bool &is_write) noexcept;
  ::std::uint64_t get_cycle() const noexcept;
  ~SimpleDDR4() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_SimpleDDR4

extern "C" {
::std::size_t cxxbridge1$SimpleDDR4$operator$sizeof() noexcept;
::std::size_t cxxbridge1$SimpleDDR4$operator$alignof() noexcept;

void cxxbridge1$init_logger() noexcept;

::SimpleDDR4 *cxxbridge1$new_ddr4(::rust::Str config) noexcept;

void cxxbridge1$SimpleDDR4$tick_ddr4(::SimpleDDR4 &self) noexcept;

bool cxxbridge1$SimpleDDR4$try_send_addr(::SimpleDDR4 &self, ::std::uint64_t addr, bool is_write) noexcept;

bool cxxbridge1$SimpleDDR4$try_recv_addr(::SimpleDDR4 &self, ::std::uint64_t &addr, bool &is_write) noexcept;

::std::uint64_t cxxbridge1$SimpleDDR4$get_cycle(const ::SimpleDDR4 &self) noexcept;
} // extern "C"

::std::size_t SimpleDDR4::layout::size() noexcept {
  return cxxbridge1$SimpleDDR4$operator$sizeof();
}

::std::size_t SimpleDDR4::layout::align() noexcept {
  return cxxbridge1$SimpleDDR4$operator$alignof();
}

void init_logger() noexcept {
  cxxbridge1$init_logger();
}

::rust::Box<::SimpleDDR4> new_ddr4(::rust::Str config) noexcept {
  return ::rust::Box<::SimpleDDR4>::from_raw(cxxbridge1$new_ddr4(config));
}

void SimpleDDR4::tick_ddr4() noexcept {
  cxxbridge1$SimpleDDR4$tick_ddr4(*this);
}

bool SimpleDDR4::try_send_addr(::std::uint64_t addr, bool is_write) noexcept {
  return cxxbridge1$SimpleDDR4$try_send_addr(*this, addr, is_write);
}

bool SimpleDDR4::try_recv_addr(::std::uint64_t &addr, bool &is_write) noexcept {
  return cxxbridge1$SimpleDDR4$try_recv_addr(*this, addr, is_write);
}

::std::uint64_t SimpleDDR4::get_cycle() const noexcept {
  return cxxbridge1$SimpleDDR4$get_cycle(*this);
}

extern "C" {
::SimpleDDR4 *cxxbridge1$box$SimpleDDR4$alloc() noexcept;
void cxxbridge1$box$SimpleDDR4$dealloc(::SimpleDDR4 *) noexcept;
void cxxbridge1$box$SimpleDDR4$drop(::rust::Box<::SimpleDDR4> *ptr) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge1 {
template <>
::SimpleDDR4 *Box<::SimpleDDR4>::allocation::alloc() noexcept {
  return cxxbridge1$box$SimpleDDR4$alloc();
}
template <>
void Box<::SimpleDDR4>::allocation::dealloc(::SimpleDDR4 *ptr) noexcept {
  cxxbridge1$box$SimpleDDR4$dealloc(ptr);
}
template <>
void Box<::SimpleDDR4>::drop() noexcept {
  cxxbridge1$box$SimpleDDR4$drop(this);
}
} // namespace cxxbridge1
} // namespace rust
