<div class="oranda-hide">

# PromptBox

</div>

PromptBox allows maintaining libraries of LLM prompt templates which can be filled in and submitted from the command
line. It can submit prompts to various hosts, including [Together](https://together.ai),  [Ollama](https://ollama.ai),
and anything compatible with the OpenAI API.

# Template Files

- are built in TOML
- can use [Tera](https://keats.github.io/tera/docs) templating (similar to Jinja), and reference templates in other files
- define command-line arguments, which can include references to files
- have the filename format `<NAME>.pb.toml`

```toml
# File: summarize.pb.toml

description = "Summarize some files"

# Optional system prompt
# Or `system_prompt_path` to read from a template file
system_prompt = "You are a great summarizer."

# This can also be template_path to read from another file.
template = '''
Create a {{style}} summary of the below files
which are on the topic of {{topic}}. The summary should be about {{ len }} sentences long.

{% for f in file -%}
File {{ f.filename }}:
{{ f.contents }}


{%- endfor %}
'''

[model]
# These model options can also be defined in a config file to apply to the whole directory of templates.
model = "gpt-3.5-turbo"
temperature = 0.7
# Also supports top_p, frequency_penalty, presence_penalty, stop, and max_tokens
# And format = "json"

[options]
len = { type = "int", description = "The length of the summary", default = 4 }
topic = { type = "string", description = "The topic of the summary" }
style = { type = "string", default = "concise" }
file = { type = "file", array = true, description = "The files to summarize" }
# For multimodal models
image = { type = "image", array = true, description = "The images to summarize" }
```

Image arguments for multimodal models will be automatically added to the request, and do not have to be referenced in the prompt template.

Then to run it:

```
> promptbox run summarize --topic software --file README.md
The README.md file provides an overview of the PromptBox utility, which is used for maintaining libraries of
LLM prompt templates that can be filled in and submitted from the command line. It explains that template files
are built in TOML and can use Tera templating. The file also includes an example template for summarizing files
on a specific topic, with options for length, formality, and the files to summarize. Additionally, it mentions the
presence of configuration files that can set default model options and inherit settings from parent directories.

> promptbox run summarize --topic software --file README.md --style excited 
Introducing PromptBox, a powerful utility for maintaining libraries of LLM prompt templates! With PromptBox, you can
easily fill in and submit prompt templates from the command line. These template files, built in TOML, can utilize
Tera templating and reference templates in other files. They also define command-line arguments, including references
to files. The excitement doesn't stop there! PromptBox even supports configuration files, allowing you to set default
model options and inherit settings from parent directories. Get ready to revolutionize your software experience
with PromptBox!
```

## Additional Input

Promptbox can take additional input from extra command-line arguments or have it piped in from another command.

`cat "transcript.txt" | pb run summarize "Here is the transcript:"`

By default, this content is appended to the end of the prompt, but the template can reference it as `{{extra}}`
to have it placed elsewhere in the prompt, as in this example. 
```jinja
Below is a transcript of a video named "{{title}}":

{{extra}}

Create a detailed outline of the above transcript.
```
This can be help when using this mode with models that work best when
their instructions are at end of the prompt.

## Model Choice

### Host Selection

PromptBox supports a few model hosts out of the box:

- lm-studio
- ollama
- openai
- openrouter
- together

While the host can be chosen explicitly, PromptBox will attempt to choose a host based on the model name using this
logic:

1. Any model name starting with "gpt-3.5" or "gpt-4" will choose OpenAI.
2. The value "lm-studio" will result in a call to LM Studio. LM Studio's API currently does not support selecting a
    model, so you will need to switch it yourself in the GUI.
3. Any other model name indicates uses the default model, which is Ollama if not otherwise configured.

See the end of this README for instructions on how to define your own hosts.

The default choice of host can be overridden by specifying the model as an object which includes the host to use:

```toml
model = { model = "mistralai/Mistral-7B-v0.1", host = "together" }
```

### Aliases

Models can use aliases as well. In either the template or a configuration file, you can add an `model.alias` section.

```toml
[model.alias]
phind = "phind-codellama:34b-v2-q5_K_M"
deepseek = "deepseek-coder:7b"
together-mistral = { model = "mistralai/Mistral-7B-v0.1", host = "together" }
```

These model aliases can then be used in place of the actual model name.


## Context Length Management

When your prompts and their input start to get large, PromptBox will trim them down to fit. There are
a few options to better control the trimming behavior.

```toml
[model.context]
# Override the context length limit from the model. Usually you can omit this unless you
# want to artificially decrease the context length to save time, money, etc.
limit = 384

# Make sure the context has enough room for this many tokens of output.
# This defaults to 256 if not otherwise specified.
# The prompt will contain roughly `limit - reserve_output` tokens.
reserve_output = 256

# When trimming context, should it keep the "start" or the "end"
keep = "start"
# keep = "end"

# The names of arguments to trim context from. If omitted, the entire prompt is trimmed to fit.
trim_args = ["extra", "files"]

# When trimming array arguments, whether to preserve the first arguments,
# the last arguments, or try to trim equally.
array_priority = "first"
# array_priority = "last"
# array_priority = "equal"
```

Currently the Llama 2 tokenizer is used regardless of the model chosen. This won't give exact results for
every model, but will be close enough for most cases.

# Configuration Files

Each directory of templates contains a configuration file, which can set default model options. Configuration files are read
from the current directory up through the parent directories. 

In each directory searched, PromptBox will look for a configuration file in that directory and in a
`promptbox` subdirectory.

The global configuration directory such as `.config/promptbox/promptbox.toml` is read as well.

A configuration file inherits settings from the configuration files in its parent directories as well, for those options that
it does not set itself. All settings in a configuration file are optional.

```toml
# By default the templates are in the same directory as the configuration file, but this can be overridden
# by setting the templates option
templates = ["template_dir"]

# This can be set to true to tell PromptBox to stop looking in parent directories for
# configurations and templates.
top_level = false

# Set this to false to tell PromptBox to not read the global configuration file.
use_global_config = true

# Use this host for models that aren't otherwise specified and aren't "lm-studio" or a GPT-3.5/4 model.
default_host = "ollama"

[model]
# Set a default model. All the other options from the template's `model` section can be used here.
model = "gpt-3.5-turbo"
```

## Custom Hosts

In addition to the built-in hosts, PromptBox supports adding additional hosts using this format in the configuration
file:

```toml
[host.my_custom_host]
# The base URL to use for the API.
endpoint = "https://super-fast-llm.example.com/api"

# protocol can be openai, ollama, or together
protocol = "openai"

# Whether or not PromptBox should limit the context length sent to the host.
# Some hosts do not provide good information on this, or have their own methods of context
# compression.
limit_context_length = true

# The name of the environment variable that holds the API key. To promote good security hygiene,
# it is not possible to embed the key directly in the configuration file.
# This can be omitted if an API key is not required for the host.
api_key = "MY_HOST_API_KEY"
```

The custom host can then be used by setting `default_host = "my_custom_host"` or by setting the host on individual models,
as described above.

### Modifying Built-In hosts

This syntax can also be used to change the behavior of built-in hosts. For example, this would change the endpoint used
for the Ollama host:

```toml
[host.ollama]
endpoint = "http://localhost:12345"
```

