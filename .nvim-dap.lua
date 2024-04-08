local dap = require('dap')

dap.configurations.rust = {
  {
    name = "Cache Path",
    type = "codelldb",
    request = "launch",
    program = function()
      return vim.fn.getcwd() .. "/target/debug/aws-credentials-cli"
    end,
    args = {
      "cache",
    },
    cwd = "${workspaceFolder}",
    stopOnEntry = false,
  },
  {
    name = "Assume Role on Account",
    type = "codelldb",
    request = "launch",
    program = function()
      return vim.fn.getcwd() .. "/target/debug/aws-credentials-cli"
    end,
    args = {
      "-a",
      function ()
        return vim.fn.input("Account: ")
      end,
      "-r",
      function ()
        return vim.fn.input("Role: ")
      end,
    },
    cwd = "${workspaceFolder}",
    stopOnEntry = false,
  },
}
