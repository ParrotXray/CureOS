#ifndef _LUNAIX_IO_H
#define _LUNAIX_IO_H

#include <stdint.h>

/**
 * @brief 向IO端口寫入一個字節
 * 
 * @param port 端口號
 * @param value 要寫入的值
 */
void io_port_wb(uint8_t port, uint8_t value);

/**
 * @brief 向IO端口寫入一個雙字
 * 
 * @param port 端口號
 * @param value 要寫入的值
 */
void io_port_wl(uint8_t port, uint32_t value);

/**
 * @brief 從IO端口讀取一個字節
 * 
 * @param port 端口號
 * @return uint8_t 讀取到的值
 */
uint8_t io_port_rb(uint8_t port);

/**
 * @brief 從IO端口讀取一個雙字
 * 
 * @param port 端口號
 * @return uint32_t 讀取到的值
 */
uint32_t io_port_rl(uint8_t port);

#endif /* _LUNAIX_IO_H */