import argparse
import json
import logging
from pathlib import Path
from typing import Dict, Tuple

import torch
import torch.nn as nn
import torch.utils.data as data
from tqdm import tqdm
from pathlib import Path
from typing import Dict, Tuple

import torch.utils.data as data
from torchvision import transforms as tfs
from torchvision.datasets import MNIST

def load_mnist(
    path_to_data: Path, batch_size: int
) -> Tuple[data.DataLoader, data.DataLoader]:
    mean = [0.5]
    std = [0.5]
    transforms = tfs.Compose([tfs.ToTensor(), tfs.Normalize(mean, std)])

    mnist_train = MNIST(path_to_data, train=True, transform=transforms, download=False)
    mnist_test = MNIST(path_to_data, train=False, transform=transforms, download=False)

    train_loader = data.DataLoader(mnist_train, batch_size=batch_size, shuffle=True)
    test_loader = data.DataLoader(mnist_test, batch_size=batch_size)

    return train_loader, test_loader


class MNIST_CNN(nn.Module):
    def __init__(self):
        super(MNIST_CNN, self).__init__()

        self.conv1 = nn.Conv2d(1, 32, kernel_size=5)
        self.conv2 = nn.Conv2d(32, 64, kernel_size=5)

        self.relu = nn.ReLU(True)
        self.pool = nn.MaxPool2d(kernel_size=2)
        self.fc1 = nn.Linear(64 * 4 * 4, 10)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = self.relu(self.pool(self.conv1(x)))
        x = self.relu(self.pool(self.conv2(x)))
        x = x.view(-1, 64 * 4 * 4)
        x = self.fc1(x)

        return nn.functional.log_softmax(x, dim=1)

def eval(
    model: nn.Module,
    data_loader: data.DataLoader,
    device: torch.device,
    log: logging.Logger,
) -> Dict:
    log.info("Testing...")

    classes = 10
    results = {}

    correct = 0
    total = 0
    class_correct = list(0.0 for _ in range(classes))
    class_total = list(0.0 for _ in range(classes))

    model.to(device)
    model.eval()

    for _, (batch_x, batch_y) in enumerate(
        tqdm(data_loader, unit="batches", desc="Testing...")
    ):
        batch_x = batch_x.to(device)
        batch_y = batch_y.to(device)

        ypred = model(batch_x)
        _, predicted = torch.max(ypred.data, 1)

        total += batch_y.size(0)
        correct += (predicted == batch_y).sum().item()

        c = predicted == batch_y.squeeze()
        for i in range(batch_y.shape[0]):
            label = batch_y[i]
            class_correct[label] += c[i].item()
            class_total[label] += 1

    accuracy = 100 * correct / total
    results["average_accuracy"] = accuracy
    for i in range(classes):
        accuracy = 100 * class_correct[i] / (class_total[i] + 0.0001)
        results[f"class_{i}_accuracy"] = accuracy

    return results


def save_results_and_conf(
    results: Dict, conf: Dict, output_dir: Path
) -> Tuple[Path, Path]:
    results_path = output_dir / "eval_results.json"
    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=4)

    if not results_path.exists():
        raise IOError("failed to save the results")

    conf_path = output_dir / "eval_conf.json"
    with open(conf_path, "w", encoding="utf-8") as f:
        json.dump(conf, f, ensure_ascii=False, indent=4)

    if not conf_path.exists():
        raise IOError("failed to save the config")

    return results_path, conf_path


def load_to_device(path_to_model: Path) -> nn.Module:
    loaded_model = MNIST_CNN()

    try:
        weights_dict = torch.load(path_to_model)
        loaded_model.load_state_dict(weights_dict)
    except:
        weights_dict = torch.load(path_to_model, map_location=torch.device("cpu"))
        loaded_model.load_state_dict(weights_dict)

    return loaded_model


def create_dir_if_doesnt_exist(path_to_dir: Path):
    path = Path(path_to_dir)
    if not path.exists():
        log.warning(f"{path_to_dir} does not exist. Creating...")
        path.mkdir(parents=True, exist_ok=True)


if __name__ == "__main__":
    logging.basicConfig(level=logging.NOTSET)
    log = logging.getLogger("TRAIN")

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path_to_data",
        type=str,
        default="./output/data",
        help="Path to store/load the data",
    )
    parser.add_argument(
        "--path_to_model",
        type=str,
        default="./output/train/model.pkl",
        help="Path to load the model",
    )
    parser.add_argument(
        "--path_to_output",
        type=str,
        default="./output/eval",
        help="Path to save output",
    )
    parser.add_argument(
        "--batch_size", type=int, default=128, help="Training batch size"
    )
    parser.add_argument(
        "--use_cuda", type=bool, default=False, help="Use CUDA for training"
    )

    args = parser.parse_args()
    conf = vars(args)

    if conf["use_cuda"]:
        if not torch.cuda.is_available():
            log.warning("use_cuda==True but no cuda devices found; will run on CPU")
            device = torch.device("cpu")
        else:
            device = torch.device("cuda")
    else:
        device = torch.device("cpu")

    _, test_loader = load_mnist(
        path_to_data=conf["path_to_data"], batch_size=conf["batch_size"]
    )

    loaded_model = load_to_device(conf["path_to_model"])

    eval_dict = eval(
        model=loaded_model, data_loader=test_loader, device=device, log=log
    )

    create_dir_if_doesnt_exist(Path(conf["path_to_output"]).resolve())
    results_path, conf_path = save_results_and_conf(
        results=eval_dict,
        conf=conf,
        output_dir=Path(conf["path_to_output"]).resolve(),
    )
    log.info(f"Results saved in {results_path}")
    log.info(f"Config saved in {conf_path}")
