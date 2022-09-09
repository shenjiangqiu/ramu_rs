
#include <iostream>
#include <ramu_rs_cpp.h>
int main() {
  init_logger();
  auto memory = new_ddr4("ddr4config.toml");
  memory->try_send_addr(10, false);
  for (auto i = 0; i < 100; i++) {
    memory->tick_ddr4();
  }
  uint64_t ret = 0;
  auto write = false;
  auto is_ret = memory->try_recv_addr(ret, write);
  if (is_ret) {
    std::cout << "ret: " << ret << " write: " << write << std::endl;
  }
}